use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use serde::{Deserialize, Serialize};

const API_URL: &str = "http://localhost:3000";

#[derive(Default, Deserialize, Serialize, Clone)]
struct Word {
    word: String,
    definition: String,
    pronunciation: String,
}

async fn get_word(dict_lang: &str, word_type: &str) -> Result<Word, String> {
    let uri = format!(
        "{API_URL}/{}/{}",
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
fn Header() -> impl IntoView {
    view! {
        <h1 class="py-4 text-2xl font-bold text-center" aria-label="header title with API name">
            "Random Words API Demo Landing Page"
        </h1>
    }
}

#[component]
fn Landing() -> impl IntoView {
    view! {
        <div class="flex flex-col h-4/5" aria-label="main landing page area">
            <div class="my-4 mt-20 border-2 hero bg-base-200 border-base-300">
                <div class="text-lg text-center hero-content">
                    <div class="max-w-lg">
                        <h1 class="text-3xl font-bold">"Simple Demo"</h1>
                        <p class="py-6">
                            "A simple landing page for my toy project "
                            <a
                                href="https://github.com/andreacfromtheapp/random-word-api"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link"
                            >
                                "Words API"
                            </a>
                            ". Which served me to learn "
                            <a
                                href="https://restfulapi.net/"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link"
                            >
                                "RESTful API"
                            </a> " development with "
                            <a
                                href="https://github.com/tokio-rs/axum"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link"
                            >
                                "Axum"
                            </a> "."
                        </p>
                        <p>
                            "My "
                            <a
                                href="https://speak-and-spell.netlify.app/"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link"
                            >
                                "Speak and Spell clone"
                            </a> " made with "
                            <a
                                href="https://elm-lang.org"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link"
                            >
                                "Elm"
                            </a> ", uses it as backend."
                        </p>
                    </div>
                </div>
            </div>

            <Demo />

            <div class="mt-12">
                <h3 class="text-2xl font-bold text-center">"OpenAPI Documentation"</h3>
                <div class="flex mt-5">
                    <ul class="flex mx-auto">
                        <li class="pr-2">
                            <button class="btn btn-neutral">
                                <a href="/swagger-ui">"Swagger UI"</a>
                            </button>
                        </li>
                        <li class="px-2">
                            <button class="btn btn-neutral">
                                <a href="/scalar">"Scalar"</a>
                            </button>
                        </li>
                        <li class="px-2">
                            <button class="btn btn-neutral">
                                <a href="/redoc">"Redoc"</a>
                            </button>
                        </li>
                        <li class="pl-2">
                            <button class="btn btn-neutral">
                                <a href="/rapidoc">"RapiDoc"</a>
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Demo() -> impl IntoView {
    let (gramm_type, set_gramm_type) = signal("Random");
    let (dict_lang, _set_dict_lang) = signal("en");
    let (word_data, set_word_data) = signal(None::<Word>);
    let (error, set_error) = signal(None::<String>);

    provide_context(set_gramm_type);

    let fetch_word = move |_| {
        let word_type = gramm_type.get();
        let lang = dict_lang.get();
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
        <div class="flex flex-col h-3/5" aria-label="main area to display a random word">
            <div class="justify-items-start py-6 px-4 m-auto w-full text-lg border-2 bg-base-200 border-base-300">
                {move || {
                    if let Some(err) = error.get() {
                        view! {
                            <div class="">
                                <p class="p-1 text-error">"Error: " {err}</p>
                            </div>
                        }
                            .into_any()
                    } else if let Some(word) = word_data.get() {
                        view! {
                            <div class="">
                                <p class="p-1">"word: " {word.word.clone()}</p>
                            </div>
                            <div class="">
                                <p class="p-1">"definition: " {word.definition.clone()}</p>
                            </div>
                            <div class="">
                                <p class="p-1">"pronunciation: " {word.pronunciation.clone()}</p>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="text-center">
                                <p class="p-1">
                                    "Load a random word (optionally choose a grammatical type first)..."
                                </p>
                            </div>
                        }
                            .into_any()
                    }
                }} <div class="justify-self-end mt-8">
                    <div class="dropdown">
                        <div tabindex="0" role="button" class="m-1 btn btn-outline">
                            Choose Type
                            <Icon icon=i::FaChevronDownSolid />
                        </div>
                        <ul
                            tabindex="0"
                            class="p-2 w-52 shadow-sm dropdown-content menu bg-base-100 rounded-box z-1"
                        >
                            <GrammaticalTypeButton btn_name="Random" />
                            <GrammaticalTypeButton btn_name="Adjective" />
                            <GrammaticalTypeButton btn_name="Adverb" />
                            <GrammaticalTypeButton btn_name="Article" />
                            <GrammaticalTypeButton btn_name="Conjunction" />
                            <GrammaticalTypeButton btn_name="Interjection" />
                            <GrammaticalTypeButton btn_name="Noun" />
                            <GrammaticalTypeButton btn_name="Preposition" />
                            <GrammaticalTypeButton btn_name="Pronoun" />
                            <GrammaticalTypeButton btn_name="Verb" />
                        </ul>
                    </div>
                    <button class="btn btn-neutral" on:click=fetch_word>
                        "Load "
                        {gramm_type}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn GrammaticalTypeButton(btn_name: &'static str) -> impl IntoView {
    let setter =
        use_context::<WriteSignal<&'static str>>().expect("to have found the setter provided");

    view! {
        <li>
            <button onclick="document.activeElement.blur()" on:click=move |_| setter.set(btn_name)>
                {btn_name}
            </button>
        </li>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer
            class="items-center p-4 footer sm:footer-horizontal"
            aria-label="footer with copyright info and link to GitHub repository"
        >
            <aside class="grid-flow-col items-center">
                <Icon icon=i::FaCopyrightRegular />
                <p>2025 - Andrea C</p>
            </aside>
            <nav class="grid-flow-col gap-2 text-2xl md:justify-self-end md:place-self-center">
                <a
                    href="https://github.com/andreacfromtheapp/random-word-api"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    <Icon icon=i::FaGithubBrands />
                </a>
            </nav>
        </footer>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="overflow-hidden overscroll-none font-sans bg-base-100 text-base-content h-dvh">
            <div class="flex flex-col m-auto max-w-3xl h-full">
                <div class="h-19/20">
                    <Header />
                    <Landing />
                </div>
                <Footer />
            </div>
        </main>
    }
}
