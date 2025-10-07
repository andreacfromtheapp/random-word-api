use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::model::*;

async fn get_word(dict_lang: &str, word_type: &str) -> Result<Word, String> {
    let uri = format!(
        "{}/{}/{}",
        api_url(),
        dict_lang.to_lowercase(),
        word_type.to_lowercase()
    );
    let response = reqwest::get(&uri).await.map_err(|e| e.to_string())?;

    match response.json::<Vec<Word>>().await {
        Ok(words) => {
            if let Some(word) = words.into_iter().next() {
                Ok(word)
            } else {
                Err("No words found in response".to_string())
            }
        }
        Err(e) => Err(format!("Failed to parse JSON: {}", e)),
    }
}

#[component]
fn Navbar() -> impl IntoView {
    view! {
        <div class="shadow-xs navbar bg-base-100">
            <div class="flex-1">
                <a href="/" class="btn btn-sm btn-ghost md:btn-md">
                    <h1 class="text-xl md:text-2xl" aria-label="header title with API name">
                        "Random Words API Demo"
                    </h1>
                </a>
            </div>
            <div class="flex-none">
                <label class="toggle text-base-content">
                    <input type="checkbox" value="dark" class="theme-controller" />
                    <Icon icon=i::FaSunSolid />
                    <Icon icon=i::FaMoonSolid />
                </label>
            </div>
        </div>
    }
}

#[component]
fn Landing() -> impl IntoView {
    view! {
        <div class="flex flex-col px-6 mt-8" aria-label="main landing page area">
            <div class="py-2">
                <h2 class="text-lg font-semibold md:text-xl">"What This Is"</h2>
                <p>
                    "A simple demo for a "
                    <a
                        href="https://restfulapi.net/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "RESTful API"
                    </a> " built with "
                    <a
                        href="https://github.com/tokio-rs/axum"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "Axum"
                    </a> " in "
                    <a
                        href="https://rust-lang.org/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "Rust"
                    </a> ", created as a personal project to dive deep into web service development
                    and to learn a number of techniques, concepts, and best practices. Initially designed to support my "
                    <a
                        href="https://github.com/andreacfromtheapp/elm_speakandspell"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "Speak and Spell"
                    </a> " application, it evolved into a "
                    <a
                        href="https://funzen.xyz/projects/random-word-api"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "comprehensive learning experience"
                    </a> ". Landing page made with "
                    <a
                        href="https://leptos.dev"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "Leptos"
                    </a> " and "
                    <a
                        href="https://daisyui.com/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "daisyUI"
                    </a> "."
                </p>
                <h2 class="mt-2 text-lg font-semibold md:text-xl">"A Note"</h2>
                <p>
                    "Don't be deceived by the minimalism of the API. Keeping the
                    model simple by design, while fulfilling all app's requirements, allowed for a
                    broader learning scope. Try out the demo below and, if more technically inclined, browse the fullstack code on the "
                    <a
                        href="https://github.com/andreacfromtheapp/random-word-api"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "Random Word API"
                    </a>
                    " repository. I hope you like it as much as I loved learning with this project! ❤️🦀"
                </p>
            </div>

            <div class="py-2">
                <h2 class="text-lg font-semibold md:text-xl">"The Demo"</h2>
                <div role="alert" class="my-2 alert alert-info alert-soft">
                    <Icon icon=i::FaCircleExclamationSolid />
                    <span class="text-base md:text-lg">
                        "Only English currently implemented, however,
                         the codebase is future-proof. Ready to accommodate more languages and grammatical types.
                         It would only need the corresponding database tables, data entry, and minor code changes."
                    </span>
                </div>

                <Demo />
            </div>

            <div class="mt-2">
                <h2 class="text-lg font-semibold md:text-xl">"API Documentation"</h2>
                <p>
                    <a
                        href="https://www.openapis.org"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "OpenAPI"
                    </a>
                    " is a standardized specification for describing RESTful APIs, providing a
                    machine-readable interface that generates interactive documentation. It transforms
                    complex API interactions into an accessible, self-documenting resource. I have
                    implemented - with the "
                    <a
                        href="https://crates.io/crates/utoipa"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary"
                    >
                        "utoipa"
                    </a>
                    " family of crates - the following OpenAPI tools:"
                </p>
                <div class="flex mt-5">
                    <ul class="flex mx-auto">
                        <li class="pr-1">
                            <button class="btn btn-sm btn-outline md:btn-md">
                                <a href="/swagger-ui">"SwaggerUI"</a>
                            </button>
                        </li>
                        <li class="px-1">
                            <button class="btn btn-sm btn-outline md:btn-md">
                                <a href="/scalar">"Scalar"</a>
                            </button>
                        </li>
                        <li class="px-1">
                            <button class="btn btn-sm btn-outline md:btn-md">
                                <a href="/redoc">"Redoc"</a>
                            </button>
                        </li>
                        <li class="pl-1">
                            <button class="btn btn-sm btn-outline md:btn-md">
                                <a href="/rapidoc">"RapiDoc"</a>
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy)]
struct GrammTypeSetter(WriteSignal<GrammaticalType>);

#[derive(Clone, Copy)]
struct DictLangSetter(WriteSignal<LanguageCode>);

#[component]
fn Demo() -> impl IntoView {
    let (gramm_type, set_gramm_type) = signal(GrammaticalType::Random);
    let (dict_lang, set_dict_lang) = signal(LanguageCode::English);
    let (word_data, set_word_data) = signal(None::<Word>);
    let (error, set_error) = signal(None::<String>);

    provide_context(GrammTypeSetter(set_gramm_type));
    provide_context(DictLangSetter(set_dict_lang));

    // Load random word on page load
    Effect::new(move |_| {
        let word_type = gramm_type.get().api_name();
        let lang = dict_lang.get().code();
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match get_word(lang, word_type).await {
                Ok(word) => {
                    set_word_data.set(Some(word));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    });

    let fetch_word = move |_| {
        let word_type = gramm_type.get().api_name();
        let lang = dict_lang.get().code();
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match get_word(lang, word_type).await {
                Ok(word) => {
                    set_word_data.set(Some(word));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col" aria-label="main area to display a random word">
            <div class="justify-items-start p-4 m-auto w-full border-2 bg-base-200 border-base-300">
                {move || {
                    match (error.get(), word_data.get()) {
                        (Some(err), _) => {
                            view! {
                                <div>
                                    <p class="p-1 text-error">"Error: " {err}</p>
                                </div>
                            }
                                .into_any()
                        }
                        (None, Some(word)) => {
                            view! {
                                <div>
                                    <p class="p-1">"word: " {word.word.clone()}</p>
                                </div>
                                <div>
                                    <p class="p-1">"definition: " {word.definition.clone()}</p>
                                </div>
                                <div>
                                    <p class="p-1">
                                        "pronunciation: " {word.pronunciation.clone()}
                                    </p>
                                </div>
                            }
                                .into_any()
                        }
                        (None, None) => {
                            view! {
                                <div role="alert" class="my-2 alert alert-error alert-soft">
                                    <Icon icon=i::FaCircleExclamationSolid />
                                    <span>
                                        "EPIC FAIL: this should NOT have happened.... d'oh!!!"
                                    </span>
                                </div>
                            }
                                .into_any()
                        }
                    }
                }} <div class="justify-self-end mt-8">
                    <div class="dropdown">
                        <div
                            tabindex="0"
                            role="button"
                            class="ml-1 btn btn-sm btn-outline md:btn-md"
                        >
                            {move || {
                                let lang = dict_lang.get();
                                let flag_class = format!("fi fi-{} fis mr-2", lang.flag_code());
                                view! {
                                    <span class=flag_class></span>
                                    <span class="hidden sm:inline">{lang.name()}</span>
                                }
                            }}
                            <Icon icon=i::FaChevronDownSolid />
                        </div>
                        <ul
                            tabindex="0"
                            class="p-2 w-52 shadow-sm dropdown-content menu bg-base-100 rounded-box z-1"
                        >
                            {LanguageCode::all()
                                .iter()
                                .map(|&lang| {
                                    view! { <LanguageCodeButton lang=lang /> }
                                })
                                .collect::<Vec<_>>()}
                        </ul>
                    </div>
                    <div class="dropdown">
                        <div
                            tabindex="1"
                            role="button"
                            class="ml-1 btn btn-sm btn-outline md:btn-md"
                        >
                            {move || gramm_type.get().name()}
                            <Icon icon=i::FaChevronDownSolid />
                        </div>
                        <ul
                            tabindex="1"
                            class="p-2 w-52 shadow-sm dropdown-content menu bg-base-100 rounded-box z-1"
                        >
                            {GrammaticalType::all()
                                .iter()
                                .map(|&g_type| {
                                    view! { <GrammaticalTypeButton g_type=g_type /> }
                                })
                                .collect::<Vec<_>>()}
                        </ul>
                    </div>
                    <button class="ml-1 btn btn-sm btn-outline md:btn-md" on:click=fetch_word>
                        "New "
                        {move || gramm_type.get().name()}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LanguageCodeButton(lang: LanguageCode) -> impl IntoView {
    let DictLangSetter(setter) = use_context().expect("DictLangSetter context");
    let flag_class = format!("fi fi-{} fis", lang.flag_code());

    view! {
        <li>
            <button onclick="document.activeElement.blur()" on:click=move |_| setter.set(lang)>
                <span class=flag_class></span>
                {lang.name()}
            </button>
        </li>
    }
}

#[component]
fn GrammaticalTypeButton(g_type: GrammaticalType) -> impl IntoView {
    let GrammTypeSetter(setter) = use_context().expect("GrammTypeSetter context");

    view! {
        <li>
            <button onclick="document.activeElement.blur()" on:click=move |_| setter.set(g_type)>
                {g_type.name()}
            </button>
        </li>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer
            class="items-center p-4 mt-12 border-t footer border-base-300 bg-base-100 footer-horizontal"
            aria-label="footer with copyright info and link to GitHub repository and LinkedIn"
        >
            <aside class="grid-flow-col items-center">
                <Icon icon=i::FaCopyrightRegular />
                <p>2025 - Andrea C</p>
            </aside>
            <nav class="grid-flow-col gap-2 text-2xl md:justify-self-end md:place-self-center">
                <a href=REPO_URL target="_blank" rel="noopener noreferrer">
                    <Icon icon=i::FaGithubBrands />
                </a>
                <a
                    href="https://www.linkedin.com/in/andreacallea"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    <Icon icon=i::FaLinkedinBrands />
                </a>
            </nav>
        </footer>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="font-sans bg-base-100 text-base-content h-dvh">
            <div class="flex flex-col m-auto max-w-3xl">
                <div class="text-base md:text-lg">
                    <Navbar />
                    <Landing />
                </div>
                <Footer />
            </div>
        </main>
    }
}
