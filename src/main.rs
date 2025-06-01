use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use yew::prelude::*;

struct Card {
    front: &'static str,
    pinyin: &'static str,
    english: &'static str,
    example: &'static str,
    example_pinyin: &'static str,
    example_english: &'static str,
    radicals: Vec<&'static str>,
}

static ALL_DECKS: Lazy<HashMap<&'static str, Vec<Card>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "HSK3",
        vec![
            Card {
                front: "我",
                pinyin: "wǒ",
                english: "I; me",
                example: "我喜欢学习中文。",
                example_pinyin: "Wǒ xǐhuān xuéxí Zhōngwén.",
                example_english: "I like studying Chinese.",
                radicals: vec!["手"],
            },
            Card {
                front: "家",
                pinyin: "jiā",
                english: "home; family",
                example: "这是我的家。",
                example_pinyin: "Zhè shì wǒ de jiā.",
                example_english: "This is my home.",
                radicals: vec!["宀"],
            },
            Card {
                front: "爱",
                pinyin: "ài",
                english: "to love",
                example: "我爱学习新东西。",
                example_pinyin: "Wǒ ài xuéxí xīn dōngxī.",
                example_english: "I love learning new things.",
                radicals: vec!["爪", "心"],
            },
        ],
    );

    m.insert(
        "HSK4",
        vec![
            Card {
                front: "努力",
                pinyin: "nǔlì",
                english: "to strive; effort",
                example: "你得多努力才能成功。",
                example_pinyin: "Nǐ děi duō nǔlì cáinéng chénggōng.",
                example_english: "You have to work harder to succeed.",
                radicals: vec!["力", "女"],
            },
            Card {
                front: "机会",
                pinyin: "jīhuì",
                english: "opportunity",
                example: "这是一个难得的机会。",
                example_pinyin: "Zhè shì yī gè nándé de jīhuì.",
                example_english: "This is a rare opportunity.",
                radicals: vec!["口", "木"],
            },
        ],
    );

    m.insert("Favorites", Vec::new());

    m
});

#[derive(Clone)]
struct Settings {
    show_pinyin: bool,
    show_english: bool,
    show_example: bool,
    show_example_pinyin: bool,
    show_example_english: bool,
    show_radicals: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_pinyin: true,
            show_english: true,
            show_example: true,
            show_example_pinyin: true,
            show_example_english: true,
            show_radicals: true,
        }
    }
}

enum Msg {
    Flip,
    KnowThis,
    DontKnow,
    ShuffleDeck,
    ChangeDeck(String),
    TogglePinyin(bool),
    ToggleEnglish(bool),
    ToggleExample(bool),
    ToggleExamplePinyin(bool),
    ToggleExampleEnglish(bool),
    ToggleRadicals(bool),
}

struct App {
    decks: HashMap<String, Vec<Card>>,
    current_deck_name: String,
    cards: Vec<Card>,
    flipped: bool,
    settings: Settings,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Clone ALL_DECKS into owned HashMap<String, Vec<Card>>
        let mut decks_owned = HashMap::new();
        for (k, v) in ALL_DECKS.iter() {
            decks_owned.insert(k.to_string(), v.clone());
        }
        let initial_deck_name = "HSK3".to_string();
        let mut cards = decks_owned
            .get(&initial_deck_name)
            .unwrap()
            .clone();
        cards.shuffle(&mut thread_rng());
        Self {
            decks: decks_owned,
            current_deck_name: initial_deck_name,
            cards,
            flipped: false,
            settings: Settings::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Flip => {
                self.flipped = !self.flipped;
                true
            }
            Msg::ShuffleDeck => {
                self.cards.shuffle(&mut thread_rng());
                self.flipped = false;
                true
            }
            Msg::ChangeDeck(name) => {
                if &name != &self.current_deck_name {
                    self.current_deck_name = name.clone();
                    let original = self.decks.get(&name).unwrap().clone();
                    self.cards = original;
                    self.cards.shuffle(&mut thread_rng());
                    self.flipped = false;
                    true
                } else {
                    false
                }
            }
            Msg::KnowThis => {
                // Remove current card (at index 0) permanently
                if !self.cards.is_empty() {
                    self.cards.remove(0);
                }
                self.flipped = false;
                true
            }
            Msg::DontKnow => {
                // Take out first card, reinsert at random position > 0
                if self.cards.len() > 1 {
                    let card = self.cards.remove(0);
                    let len = self.cards.len();
                    let idx = thread_rng().gen_range(0..=len);
                    self.cards.insert(idx, card);
                }
                self.flipped = false;
                true
            }
            Msg::TogglePinyin(v) => {
                self.settings.show_pinyin = v;
                true
            }
            Msg::ToggleEnglish(v) => {
                self.settings.show_english = v;
                true
            }
            Msg::ToggleExample(v) => {
                self.settings.show_example = v;
                true
            }
            Msg::ToggleExamplePinyin(v) => {
                self.settings.show_example_pinyin = v;
                true
            }
            Msg::ToggleExampleEnglish(v) => {
                self.settings.show_example_english = v;
                true
            }
            Msg::ToggleRadicals(v) => {
                self.settings.show_radicals = v;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let deck_options = self
            .decks
            .keys()
            .map(|name| html! {
                <option value={name.clone()} selected={*name == self.current_deck_name}>{name.clone()}</option>
            })
            .collect::<Html>();

        let current_card = self.cards.get(0);

        html! {
            <div style="max-width: 500px; margin: auto; font-family: sans-serif;">
                <h2>{ "Chinese Flashcards (Rust + Yew)" }</h2>

                // Deck Controls
                <div style="margin-bottom: 10px;">
                    <label for="deck-select">{ "Deck: " }</label>
                    <select id="deck-select"
                        onchange={ctx.link().callback(|e: Event| {
                            let select = e.target_unchecked_into::<web_sys::HtmlSelectElement>();
                            Msg::ChangeDeck(select.value())
                        })}>
                        { deck_options }
                    </select>
                    <button onclick={ctx.link().callback(|_| Msg::ShuffleDeck)} style="margin-left: 10px;">
                        { "Shuffle" }
                    </button>
                </div>

                // Settings
                <details style="margin-bottom: 20px;">
                    <summary>{ "Settings (toggle fields)" }</summary>
                    <div style="margin-top: 5px;">
                        <input type="checkbox"
                            id="chk_pinyin"
                            checked={self.settings.show_pinyin}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::TogglePinyin(chk.checked())
                            })} />
                        <label for="chk_pinyin">{ "Pinyin" }</label>
                        <br/>

                        <input type="checkbox"
                            id="chk_english"
                            checked={self.settings.show_english}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::ToggleEnglish(chk.checked())
                            })} />
                        <label for="chk_english">{ "English" }</label>
                        <br/>

                        <input type="checkbox"
                            id="chk_example"
                            checked={self.settings.show_example}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::ToggleExample(chk.checked())
                            })} />
                        <label for="chk_example">{ "Example Sentence" }</label>
                        <br/>

                        <input type="checkbox"
                            id="chk_example_pinyin"
                            checked={self.settings.show_example_pinyin}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::ToggleExamplePinyin(chk.checked())
                            })} />
                        <label for="chk_example_pinyin">{ "Example Pinyin" }</label>
                        <br/>

                        <input type="checkbox"
                            id="chk_example_english"
                            checked={self.settings.show_example_english}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::ToggleExampleEnglish(chk.checked())
                            })} />
                        <label for="chk_example_english">{ "Example English" }</label>
                        <br/>

                        <input type="checkbox"
                            id="chk_radicals"
                            checked={self.settings.show_radicals}
                            onchange={ctx.link().callback(|e: Event| {
                                let chk = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                                Msg::ToggleRadicals(chk.checked())
                            })} />
                        <label for="chk_radicals">{ "Radicals" }</label>
                    </div>
                </details>

                // Card Display
                {
                    if let Some(card) = current_card {
                        html! {
                            <div style="border: 1px solid #ccc; padding: 20px; border-radius: 8px; text-align: center;">
                                {
                                    if !self.flipped {
                                        html! {
                                            <>
                                                <div style="font-size: 48px; margin-bottom: 20px;">{ card.front }</div>
                                                <button onclick={ctx.link().callback(|_| Msg::Flip)}>
                                                    { "Flip" }
                                                </button>
                                            </>
                                        }
                                    } else {
                                        html! {
                                            <>
                                                // Pinyin
                                                {
                                                    if self.settings.show_pinyin {
                                                        html! {
                                                            <div style="font-size: 24px; margin-bottom: 10px;">
                                                                { card.pinyin }
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // English
                                                {
                                                    if self.settings.show_english {
                                                        html! {
                                                            <div style="font-size: 18px; margin-bottom: 10px;">
                                                                { card.english }
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // Example Sentence
                                                {
                                                    if self.settings.show_example {
                                                        html! {
                                                            <div style="margin-top: 10px;">
                                                                <em>{ card.example }</em>
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // Example Pinyin
                                                {
                                                    if self.settings.show_example_pinyin {
                                                        html! {
                                                            <div style="font-size: 14px; color: gray;">
                                                                { card.example_pinyin }
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // Example English
                                                {
                                                    if self.settings.show_example_english {
                                                        html! {
                                                            <div style="font-size: 14px; color: gray; margin-bottom: 10px;">
                                                                { card.example_english }
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // Radicals
                                                {
                                                    if self.settings.show_radicals {
                                                        html! {
                                                            <div style="font-size: 14px; color: #555; margin-bottom: 20px;">
                                                                { format!("Radical(s): {}", card.radicals.join(", ")) }
                                                            </div>
                                                        }
                                                    } else { html! { <></> } }
                                                }

                                                // Buttons
                                                <div>
                                                    <button onclick={ctx.link().callback(|_| Msg::KnowThis)} style="margin-right: 10px;">
                                                        { "I know this" }
                                                    </button>
                                                    <button onclick={ctx.link().callback(|_| Msg::DontKnow)}>
                                                        { "I don't know" }
                                                    </button>
                                                </div>
                                            </>
                                        }
                                    }
                                }
                            </div>
                        }
                    } else {
                        html! {
                            <div style="padding: 20px; text-align: center;">
                                { "No more cards in this deck!"}
                            </div>
                        }
                    }
                }
            </div>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
