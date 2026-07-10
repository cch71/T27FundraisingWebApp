use tracing::error;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct ModuleHostProps {
    pub(crate) module: AttrValue,
}

/// Hosts a dynamically loaded wasm page module. The module's JS and wasm are
/// fetched on first visit and mounted into the div below; switching modules
/// unmounts the old one (its wasm instance is kept so remounts are instant).
#[component(ModuleHost)]
pub(crate) fn module_host(props: &ModuleHostProps) -> Html {
    let is_loaded = use_state_eq(|| false);

    {
        let is_loaded = is_loaded.clone();
        use_effect_with(props.module.clone(), move |module| {
            is_loaded.set(false);
            let name = module.to_string();
            {
                let name = name.clone();
                let is_loaded = is_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match js::module_loader::load_module(&name, "appModuleRoot").await {
                        Ok(_) => is_loaded.set(true),
                        Err(err) => {
                            error!("Failed to load the {name} module: {err:#?}");
                            gloo::dialogs::alert(&format!(
                                "Failed to load the {name} module: {err:#?}"
                            ));
                        }
                    }
                });
            }
            move || js::module_loader::unload_module(&name)
        });
    }

    html! {
        <>
            if !*is_loaded {
                <div class="justify-content-center text-center">
                    <span class="loader"></span>
                </div>
            }
            <div id="appModuleRoot"></div>
        </>
    }
}
