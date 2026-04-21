use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use num_bigint::BigInt;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use swc_core::common::{GLOBALS, source_map::SmallPos};
use tracing::instrument;
use turbo_rcstr::{RcStr, rcstr};
use turbo_tasks::{ResolvedVc, TryJoinIterExt, Vc};
use turbo_tasks_fs::FileSystemPath;
use turbopack_core::{
    compile_time_info::CompileTimeInfo,
    issue::{Issue, IssueExt, IssueSeverity, IssueSource, IssueStage, StyledString},
    module::Module,
    reference::ModuleReference,
};

use crate::{
    AnalyzeMode, EcmascriptParsable,
    analyzer::{
        ConstantValue, JsValue, ModuleValue, ObjectPart, builtin::replace_builtin,
        graph::create_graph, linker::link, well_known::replace_well_known,
    },
    directive::parse_module_turbopack_directives,
    parse::ParseResult,
    references::{early_value_visitor, esm::EsmAssetReference},
};

/// Import names that are all-uppercase and contain at least one letter are eligible for automatic
/// constant inlining, even without an import attribute.
pub fn is_import_name_eligible_for_exports(name: &str) -> bool {
    let mut seen_alphabetic = false;
    for c in name.chars() {
        if !(c.is_ascii() && (!c.is_ascii_alphabetic() || c.is_uppercase())) {
            return false;
        }
        seen_alphabetic |= c.is_ascii_alphabetic();
    }
    seen_alphabetic
}

#[instrument(level = "info", skip_all, name = "determine cross-module constants")]
pub async fn module_value_to_constants_module(
    module_value: &ModuleValue,
    compile_time_info: Vc<CompileTimeInfo>,
    import_references: &[ResolvedVc<EsmAssetReference>],
) -> Result<Option<JsValue>> {
    let Some(reference_idx) = module_value.reference else {
        bail!("missing reference for constant value");
    };

    let import_reference = import_references
        .get(reference_idx)
        .with_context(|| format!("couldn't find import reference at index {reference_idx}"))?;

    // We are reusing the exact resolve options from EsmAssetReference here, which is good and gives
    // us side-effect-free barrel file resolving for free.
    let resolved = import_reference.resolve_reference().await?;
    let resolved = resolved.primary_modules_ref().await?;
    let Some(module) = resolved.first() else {
        // failed to resolve, issue was already emitted by resolve_reference
        return Ok(None);
    };

    let constants = get_constants(**module, compile_time_info).await?;

    Ok(constants.as_ref().map(|constants| {
        constants.as_js_value(
            module_value
                .annotations
                .as_ref()
                .is_some_and(|a| a.has_turbopack_constants()),
        )
    }))
}

#[turbo_tasks::value]
#[derive(Debug)]
struct ConstantsModule {
    exports: Vec<(RcStr, Option<ConstantValue>)>,
    has_directive: bool,
}

#[turbo_tasks::value(transparent)]
#[derive(Debug)]
struct OptionConstantsModule(Option<ConstantsModule>);

impl ConstantsModule {
    pub fn as_js_value(&self, has_turbopack_annotation: bool) -> JsValue {
        let has_opt_in = self.has_directive || has_turbopack_annotation;

        // This has to be
        // - mutable:false, otherwise nothing would ever be inlined, because all property accesses
        //   would be receive a `|unknown` alternative
        // - frozen:true, otherwise mutable:false would cause accesses of missing properties to be
        //   `undefined`. Because we return a JsValue::Object even if the module has only some
        //   constants exports, this would cause `import {NON_CONSTANT_EXPORT}` to be incorrectly
        //   replaced with `undefined`.
        JsValue::frozen_object_missing_unknown(
            self.exports
                .iter()
                .map(|(key, value)| {
                    ObjectPart::KeyValue(
                        JsValue::Constant(ConstantValue::Str(key.clone().into())),
                        if let Some(value) = value {
                            if !has_opt_in {
                                // when not having opt in, only inline short literals
                                match value {
                                    ConstantValue::Str(s) if s.as_str().len() > 6 => {
                                        JsValue::unknown_empty(false, "constant too long")
                                    }
                                    ConstantValue::Num(n) if n.0.abs() > 1_000_000.0 => {
                                        JsValue::unknown_empty(false, "constant too long")
                                    }
                                    ConstantValue::BigInt(n)
                                        if **n > BigInt::from(1_000_000)
                                            || **n < BigInt::from(-1_000_000) =>
                                    {
                                        JsValue::unknown_empty(false, "constant too long")
                                    }
                                    ConstantValue::Regex(_) => {
                                        // Regexes are literals, but they are also objects, so
                                        // have identity and aren't inlined without opt in.
                                        JsValue::unknown_empty(false, "regex not inlined")
                                    }
                                    _ => JsValue::Constant(value.clone()),
                                }
                            } else {
                                JsValue::Constant(value.clone())
                            }
                        } else {
                            JsValue::unknown_empty(false, "not a constant")
                        },
                    )
                })
                .collect(),
        )
    }
}

#[turbo_tasks::function]
pub async fn get_constants(
    module: ResolvedVc<Box<dyn Module>>,
    compile_time_info: Vc<CompileTimeInfo>,
) -> Result<Vc<OptionConstantsModule>> {
    let Some(parseable) = ResolvedVc::try_sidecast::<Box<dyn EcmascriptParsable>>(module) else {
        // should never actually happen, there should be a "imported module is not chunkable" error
        // somewhere as well if it's truly not an Ecmascript module
        return Ok(Vc::cell(None));
    };

    let parsed = parseable.failsafe_parse().await?;
    let ParseResult::Ok {
        program,
        eval_context,
        globals,
        ..
    } = &*parsed
    else {
        // The `parse` call has already emitted parse issues in case of `ParseResult::Unparsable`
        return Ok(Vc::cell(None));
    };

    let directives = parse_module_turbopack_directives(program);

    let var_graph = {
        let supports_block_scoping = *compile_time_info
            .environment()
            .runtime_versions()
            .supports_block_scoping()
            .await?;
        let _span = tracing::trace_span!("analyze variable values").entered();
        GLOBALS.set(globals, || {
            create_graph(
                program,
                eval_context,
                AnalyzeMode::Tracing,
                supports_block_scoping,
            )
        })
    };

    let fun_args_values = Mutex::new(FxHashMap::default());
    let var_cache = Mutex::new(FxHashMap::default());

    let compile_time_info_ref = compile_time_info.await?;

    let exports: Vec<(RcStr, Option<ConstantValue>)> = eval_context
        .imports
        .exports_ids
        .iter()
        .map(async |(export_name, (binding, span))| {
            let value = GLOBALS.set(globals, || eval_context.eval_id(binding.clone()));

            let linked_value = link(
                &var_graph,
                value.clone(),
                &early_value_visitor,
                &async |v| {
                    if let Some((name, _)) = v.get_definable_name(Some(&var_graph))
                        && let Some(value) = compile_time_info_ref.defines.get(&name).await?
                    {
                        return Ok(((&*value).try_into()?, true));
                    }

                    // if directives.constants_module
                    //     && let JsValue::Module(module) = &v
                    // {
                    //     // TODO when opted in, also resolve imports
                    //     // We can't do a recursive turbotask call here, to prevent deadlocks.
                    //     if let Some(constants) =
                    //         get_constants(resolve_somehow(module), compile_time_info)
                    //             .await?
                    //             .as_ref()
                    //     {
                    //         return Ok((constants.as_js_value(false), true));
                    //     }
                    // }

                    let (mut v, mut modified) =
                        replace_well_known(v, compile_time_info, false).await?;
                    modified = replace_builtin(&mut v) || modified;
                    modified = modified || v.make_nested_operations_unknown();
                    Ok((v, modified))
                },
                &fun_args_values,
                &var_cache,
            )
            .await?;

            if let JsValue::Constant(constant) = linked_value.0 {
                Ok((export_name.as_str().into(), Some(constant)))
            } else {
                if directives.constants_module {
                    NonConstantIssue {
                        export: export_name.as_str().into(),
                        file_path: module.ident().await?.path.clone(),
                        source: module.source().await?.map(|source| {
                            IssueSource::from_swc_offsets(
                                source,
                                span.lo.to_u32(),
                                span.hi.to_u32(),
                            )
                        }),
                        value: linked_value.0.explain(10, 5),
                    }
                    .resolved_cell()
                    .emit();
                }
                Ok((export_name.as_str().into(), None))
            }
        })
        .try_join()
        .await?;

    Ok(Vc::cell(Some(ConstantsModule {
        exports,
        has_directive: directives.constants_module,
    })))
}

#[turbo_tasks::value]
struct NonConstantIssue {
    export: RcStr,
    file_path: FileSystemPath,
    source: Option<IssueSource>,
    value: (String, String),
}

#[async_trait]
#[turbo_tasks::value_impl]
impl Issue for NonConstantIssue {
    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Error
    }

    async fn title(&self) -> Result<StyledString> {
        Ok(StyledString::Line(vec![
            StyledString::Text(rcstr!("Export ")),
            StyledString::Code(self.export.clone()),
            StyledString::Text(rcstr!(" is not a constant")),
        ]))
    }

    fn stage(&self) -> IssueStage {
        IssueStage::Analysis
    }

    async fn file_path(&self) -> Result<FileSystemPath> {
        Ok(self.file_path.clone())
    }

    async fn description(&self) -> Result<Option<StyledString>> {
        Ok(Some(StyledString::Stack(vec![
            StyledString::Line(vec![
                StyledString::Text(rcstr!("It was analyzed to be ")),
                StyledString::Code(self.value.0.clone().into()),
            ]),
            StyledString::Line(vec![StyledString::Code(self.value.1.clone().into())]),
        ])))
    }

    fn source(&self) -> Option<IssueSource> {
        self.source
    }
}
