use std::collections::HashMap;
use rand::Rng;
use serde::Deserialize;
use yew::prelude::*;
use web_sys::{HtmlSelectElement, HtmlInputElement};

// -------------------------------
// 1. Define data structures
// -------------------------------

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Example {
    chinese: String,
    pinyin:  String,
    english: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Radical {
    character: String,
    pinyin:    String,
    meaning:   String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Card {
    character: String,
    pinyin:    String,
    english:   String,
    examples:  Vec<Example>,   // Three example sentences
    radicals:  Vec<Radical>,   // Each radical has character + pinyin + meaning
}

// We’ll parse the entire JSON into a HashMap<String, Vec<Card>>:
type DecksMap = HashMap<String, Vec<Card>>;


// -------------------------------
// 2. Main App Component
// -------------------------------

struct App {
    decks: DecksMap,
    current_deck: String,
    cards: Vec<Card>,
    current_index: usize,
    show_back: bool,

    // Toggles for each field on back:
    show_pinyin: bool,
    show_english: bool,
    show_examples: bool,
    show_examples_pinyin: bool,
    show_examples_english: bool,
    show_radicals: bool,
}

enum Msg {
    Flip,
    Next,
    Remove,
    Shuffle,
    SelectDeck(String),
    AddToFavorites,
    TogglePinyin(bool),
    ToggleEnglish(bool),
    ToggleExamples(bool),
    ToggleExamplesPinyin(bool),
    ToggleExamplesEnglish(bool),
    ToggleRadicals(bool),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // 1. At compile time, include the JSON file:
        let raw_json = include_str!("cards.json");
        let mut decks: DecksMap = serde_json::from_str(raw_json)
            .expect("cards.json should be valid JSON with correct structure");

        // Ensure there's a "Favorites" deck even if absent:
        if !decks.contains_key("Favorites") {
            decks.insert("Favorites".to_string(), Vec::new());
        }

        // Start with HSK1 as default:
        let current_deck = "HSK1".to_string();
        let cards = decks.get(&current_deck).unwrap().to_vec();

        Self {
            decks,
            current_deck,
            cards,
            current_index: 0,
            show_back: false,

            show_pinyin: true,
            show_english: true,
            show_examples: true,
            show_examples_pinyin: true,
            show_examples_english: true,
            show_radicals: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Flip => {
                self.show_back = !self.show_back;
                true
            }
            Msg::Next => {
                if !self.cards.is_empty() {
                    self.current_index = (self.current_index + 1) % self.cards.len();
                }
                self.show_back = false;
                true
            }
            Msg::Remove => {
                if !self.cards.is_empty() {
                    self.cards.remove(self.current_index);
                    // Also remove from the underlying deck if it's not "Favorites"
                    if self.current_deck != "Favorites" {
                        if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                            if self.current_index < deck_vec.len() {
                                deck_vec.remove(self.current_index);
                            }
                        }
                    }
                    if self.current_index >= self.cards.len() && !self.cards.is_empty() {
                        self.current_index = 0;
                    }
                }
                self.show_back = false;
                true
            }
            Msg::Shuffle => {
                let mut rng = rand::thread_rng();
                let len = self.cards.len();
                for i in (1..len).rev() {
                    let j = rng.gen_range(0..=i);
                    self.cards.swap(i, j);
                }
                self.current_index = 0;
                self.show_back = false;
                true
            }
            Msg::SelectDeck(name) => {
                if let Some(deck_cards) = self.decks.get(&name) {
                    self.current_deck = name.clone();
                    self.cards = deck_cards.clone();
                    self.current_index = 0;
                    self.show_back = false;
                    true
                } else {
                    false
                }
            }
            Msg::AddToFavorites => {
                if let Some(card) = self.cards.get(self.current_index).cloned() {
                    // Check if it already exists
                    let fav_deck = self.decks.get_mut("Favorites").unwrap();
                    let already = fav_deck.iter().any(|c| c.character == card.character);
                    if !already {
                        fav_deck.push(card);
                    }
                }
                false
            }
            Msg::TogglePinyin(val) => {
                self.show_pinyin = val;
                true
            }
            Msg::ToggleEnglish(val) => {
                self.show_english = val;
                true
            }
            Msg::ToggleExamples(val) => {
                self.show_examples = val;
                true
            }
            Msg::ToggleExamplesPinyin(val) => {
                self.show_examples_pinyin = val;
                true
            }
            Msg::ToggleExamplesEnglish(val) => {
                self.show_examples_english = val;
                true
            }
            Msg::ToggleRadicals(val) => {
                self.show_radicals = val;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Build a <select> with deck names:
        let deck_options = self.decks.keys().map(|deck_name| {
            let selected = *deck_name == self.current_deck;
            html! {
                <option value={deck_name.clone()} selected={selected}>{ deck_name.clone() }</option>
            }
        });

        // Current card (if any):
        let card_opt = self.cards.get(self.current_index);

        html! {
            <div style="padding: 1em;">
                <h1>{ "Chinese Flashcards" }</h1>

                // Deck selector + Shuffle
                <div style="margin-bottom: 1em;">
                    <label for="deck-select"><b>{ "Deck: " }</b></label>
                    <select id="deck-select"
                        onchange={ctx.link().callback(|e: Event| {
                            let sel = e.target_unchecked_into::<HtmlSelectElement>();
                            Msg::SelectDeck(sel.value())
                        })}
                    >
                        { for deck_options }
                    </select>
                    <button onclick={ctx.link().callback(|_| Msg::Shuffle)} style="margin-left: 1em;">
                        { "Shuffle" }
                    </button>
                </div>

                // Toggles
                <div style="margin-bottom: 1em;">
                    <label style="margin-right: 1em;">
                        <input type="checkbox"
                            checked={self.show_pinyin}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::TogglePinyin(chk.checked())
                            })}
                        />{ "Pinyin" }
                    </label>
                    <label style="margin-right: 1em;">
                        <input type="checkbox"
                            checked={self.show_english}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::ToggleEnglish(chk.checked())
                            })}
                        />{ "English" }
                    </label>
                    <label style="margin-right: 1em;">
                        <input type="checkbox"
                            checked={self.show_examples}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::ToggleExamples(chk.checked())
                            })}
                        />{ "Examples" }
                    </label>
                    <label style="margin-right: 1em;">
                        <input type="checkbox"
                            checked={self.show_examples_pinyin}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::ToggleExamplesPinyin(chk.checked())
                            })}
                        />{ "Examples Pinyin" }
                    </label>
                    <label style="margin-right: 1em;">
                        <input type="checkbox"
                            checked={self.show_examples_english}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::ToggleExamplesEnglish(chk.checked())
                            })}
                        />{ "Examples Eng." }
                    </label>
                    <label>
                        <input type="checkbox"
                            checked={self.show_radicals}
                            onchange={ctx.link().callback(|e:Event| {
                                let chk = e.target_unchecked_into::<HtmlInputElement>();
                                Msg::ToggleRadicals(chk.checked())
                            })}
                        />{ "Radicals" }
                    </label>
                </div>

                // Card container (fixed size so it always looks like a “card”)
                <div
                    style="
                        width: 100%;
                        max-width: 400px;
                        height: 300px;
                        margin: 0 auto;
                        border: 1px solid #ccc;
                        border-radius: 8px;
                        background-color: #fff;
                        box-shadow: 2px 2px 8px rgba(0, 0, 0, 0.1);
                        padding: 1em;
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        text-align: center;
                        overflow-y: auto;
                        cursor: pointer;
                    "
                    onclick={ctx.link().callback(|_| Msg::Flip)}
                >
                    {
                        if let Some(card) = card_opt {
                            if !self.show_back {
                                html! {
                                    <div style="font-size: 3em; line-height: 1; margin-bottom: 0.5em;">
                                        { &card.character }
                                    </div>
                                }
                            } else {
                                html! {
                                    <div>
                                        // Pinyin
                                        { if self.show_pinyin {
                                            html! {
                                                <p style="font-size: 1.2em; margin: 0.2em 0;">
                                                    <b>{ "Pinyin: " }</b>{ &card.pinyin }
                                                </p>
                                            }
                                        } else { html!{} } }

                                        // English
                                        { if self.show_english {
                                            html! {
                                                <p style="font-size: 1em; margin: 0.2em 0;">
                                                    <b>{ "English: " }</b>{ &card.english }
                                                </p>
                                            }
                                        } else { html!{} } }

                                        // Examples (three)
                                        { if self.show_examples {
                                            html! {
                                                <div style="margin-top: 0.5em; text-align: left;">
                                                    { for card.examples.iter().map(|ex| html! {
                                                        <div style="margin-bottom: 0.5em;">
                                                          <p style="margin:0;"><i>{ &ex.chinese }</i></p>
                                                          { if self.show_examples_pinyin {
                                                              html! {
                                                                <p style="margin:0; font-size:0.9em; color:gray;">
                                                                  { &ex.pinyin }
                                                                </p>
                                                              }
                                                            } else { html!{} } }
                                                          { if self.show_examples_english {
                                                              html! {
                                                                <p style="margin:0; font-size:0.9em; color:gray;">
                                                                  { &ex.english }
                                                                </p>
                                                              }
                                                            } else { html!{} } }
                                                        </div>
                                                    })}
                                                </div>
                                            }
                                        } else { html!{} } }

                                        // Radicals
                                        { if self.show_radicals {
                                            html! {
                                                <div style="margin-top: 0.5em; text-align:left;">
                                                    <b>{ "Radicals: " }</b>
                                                    { for card.radicals.iter().map(|r| html! {
                                                        <div style="margin: 0.2em 0;">
                                                          <span style="font-size:1.1em;">{ &r.character }</span>
                                                          <span style="margin-left:0.5em; font-size:0.9em;">
                                                            { "(" }{ &r.pinyin }{ " – " }{ &r.meaning }{ ")" }
                                                          </span>
                                                        </div>
                                                    })}
                                                </div>
                                            }
                                        } else { html!{} } }
                                    </div>
                                }
                            }
                        } else {
                            html! {
                                <p style="font-size:1.2em; color: #888;">
                                    { "No cards in this deck." }
                                </p>
                            }
                        }
                    }
                </div>

                // Buttons under the card
                <div style="margin-top: 1em; text-align:center;">
                    <button
                        onclick={ctx.link().callback(|_| Msg::Next)}
                        disabled={self.cards.is_empty()}
                    >
                        { "Next" }
                    </button>
                    <button
                        onclick={ctx.link().callback(|_| Msg::Remove)}
                        disabled={self.cards.is_empty()}
                        style="margin-left: 1em;"
                    >
                        { "Remove" }
                    </button>
                    <button
                        onclick={ctx.link().callback(|_| Msg::AddToFavorites)}
                        disabled={self.cards.is_empty() || self.current_deck == "Favorites"}
                        style="margin-left: 1em;"
                    >
                        { "Add to Favorites" }
                    </button>
                </div>
            </div>
        }
    }
}

// -------------------------------
// 3. Boot the App
// -------------------------------

fn main() {
    // Yew 0.20+ uses Renderer rather than start_app
    yew::Renderer::<App>::new().render();
}
