use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

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
            <div class="my-4 mt-20 hero bg-base-200">
                <div class="text-center hero-content">
                    <div class="max-w-md">
                        <h1 class="text-3xl font-bold">"Simple Demo"</h1>
                        <p class="py-6">
                            "This is a simple landing page for my toy project Random Words API.
                            This project served me well to learn developing a RESTful API with Rust
                            using Axum. The landing page is a convenience entry point for users to
                            access the different interfaces for interacting with random words."
                        </p>
                    </div>
                </div>
            </div>

            <div class="flex my-8">
                <div class="w-96 card card-border bg-base-100">
                    <div class="card-body">
                        <h2 class="card-title">"Speak and Spell Game"</h2>
                        <p>
                            "Interactive word learning game. A Speak and Spell clone Made with Elm."
                        </p>
                        <div class="justify-end card-actions">
                            <button class="btn btn-primary">
                                <a href="/play/">"Play"</a>
                            </button>
                        </div>
                    </div>
                </div>

                <div class="w-96 card card-border bg-base-100">
                    <div class="card-body">
                        <h2 class="card-title">"Random Word Generator"</h2>
                        <p>
                            "Retrieve random words from the database on demand. Made with Leptos."
                        </p>
                        <div class="justify-end card-actions">
                            <button class="btn btn-primary">
                                <a href="/random/">"Random"</a>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="mt-12">
                <h3 class="text-2xl font-bold text-center">"OpenAPI Documentation"</h3>
                <div class="flex mt-5">
                    <ul class="flex mx-auto">
                        <li class="pr-2">
                            <button class="btn btn-lg btn-primary">
                                <a href="/swagger-ui">"Swagger UI"</a>
                            </button>
                        </li>
                        <li class="px-2">
                            <button class="btn btn-lg btn-primary">
                                <a href="/scalar">"Scalar"</a>
                            </button>
                        </li>
                        <li class="px-2">
                            <button class="btn btn-lg btn-primary">
                                <a href="/redoc">"Redoc"</a>
                            </button>
                        </li>
                        <li class="pl-2">
                            <button class="btn btn-lg btn-primary">
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
