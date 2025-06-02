use std::collections::HashMap;
use rand::Rng;
use serde::Deserialize;
use yew::prelude::*;
use web_sys::{HtmlSelectElement, HtmlInputElement, HtmlElement};

//───────────────────────────────────────────────────────────────────────────────
// 1. Data structures (load from JSON)
//───────────────────────────────────────────────────────────────────────────────

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
    examples:  Vec<Example>,
    radicals:  Vec<Radical>,
}

type DecksMap = HashMap<String, Vec<Card>>;

//───────────────────────────────────────────────────────────────────────────────
// 2. Animation state enum (derive PartialEq so we can compare)
//───────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq)]
enum AnimationState {
    None,
    Removing,
    Replacing,
    Shuffling,
}

//───────────────────────────────────────────────────────────────────────────────
// 3. Main App component
//───────────────────────────────────────────────────────────────────────────────

struct App {
    decks: DecksMap,
    current_deck: String,
    cards: Vec<Card>,
    current_index: usize,
    show_back: bool,
    anim_state: AnimationState,

    // Back‐side toggles
    show_pinyin: bool,
    show_english: bool,
    show_examples: bool,
    show_examples_pinyin: bool,
    show_examples_english: bool,
    show_radicals: bool,

    // For “Removed” counter
    initial_count: usize,

    // For keyboard focus
    container_ref: NodeRef,
    focus_set: bool,
}

enum Msg {
    Flip,
    StartRemove,
    StartReplace,
    StartShuffle,
    AnimDone,
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
        // Load each deck’s JSON via include_str! macros
        let mut decks: DecksMap = HashMap::new();
        let hsk3: Vec<Card> = serde_json::from_str(include_str!("decks/HSK3.json"))
            .expect("Failed to parse decks/HSK3.json");
        let hsk4: Vec<Card> = serde_json::from_str(include_str!("decks/HSK4.json"))
            .expect("Failed to parse decks/HSK4.json");

        decks.insert("HSK3".into(), hsk3.clone());
        decks.insert("HSK4".into(), hsk4.clone());
        // Empty “Favorites”
        decks.insert("Favorites".into(), Vec::new());

        // Default to HSK3
        let current_deck = "HSK3".into();
        let cards = decks.get(&current_deck).unwrap().clone();
        let initial_count = cards.len();

        App {
            decks,
            current_deck,
            cards,
            current_index: 0,
            show_back: false,
            anim_state: AnimationState::None,

            show_pinyin: true,
            show_english: true,
            show_examples: true,
            show_examples_pinyin: true,
            show_examples_english: true,
            show_radicals: false, // off by default

            initial_count,
            container_ref: NodeRef::default(),
            focus_set: false,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        // Focus so arrow keys/shuffle “s” work immediately
        if first_render && !self.focus_set {
            if let Some(elem) = self.container_ref.cast::<HtmlElement>() {
                let _ = elem.focus();
                self.focus_set = true;
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Flip => {
                if self.anim_state == AnimationState::None {
                    self.show_back = !self.show_back;
                    true
                } else {
                    false
                }
            }

            Msg::StartRemove => {
                if self.anim_state == AnimationState::None {
                    self.anim_state = AnimationState::Removing;
                    self.show_back = false;
                    true
                } else {
                    false
                }
            }

            Msg::StartReplace => {
                if self.anim_state == AnimationState::None {
                    self.anim_state = AnimationState::Replacing;
                    self.show_back = false;
                    true
                } else {
                    false
                }
            }

            Msg::StartShuffle => {
                if self.anim_state == AnimationState::None {
                    self.anim_state = AnimationState::Shuffling;
                    true
                } else {
                    false
                }
            }

            Msg::AnimDone => {
                match std::mem::replace(&mut self.anim_state, AnimationState::None) {
                    AnimationState::Removing => {
                        if !self.cards.is_empty() {
                            // Remove from self.cards
                            let _ = self.cards.remove(self.current_index);
                            // Only remove from decks map if not Favorites
                            if self.current_deck != "Favorites" {
                                if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                                    // Only remove if the deck_vec is not the same as self.cards
                                    // (to avoid removing from all decks due to shared Vec)
                                    if !std::ptr::eq(deck_vec, &self.cards) {
                                        if self.current_index < deck_vec.len() {
                                            deck_vec.remove(self.current_index);
                                        }
                                    }
                                }
                            }
                            if !self.cards.is_empty() {
                                self.current_index %= self.cards.len();
                            }
                        }
                    }
                    AnimationState::Replacing => {
                        if !self.cards.is_empty() {
                            let card = self.cards[self.current_index].clone();
                            self.cards.remove(self.current_index);
                            if self.current_deck != "Favorites" {
                                if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                                    if !std::ptr::eq(deck_vec, &self.cards) {
                                        if self.current_index < deck_vec.len() {
                                            deck_vec.remove(self.current_index);
                                        }
                                    }
                                }
                            }
                            let mut rng = rand::thread_rng();
                            let len = self.cards.len();
                            if len == 0 {
                                self.cards.push(card.clone());
                                if self.current_deck != "Favorites" {
                                    if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                                        if !std::ptr::eq(deck_vec, &self.cards) {
                                            deck_vec.push(card);
                                        }
                                    }
                                }
                                self.current_index = 0;
                            } else {
                                let idx = rng.gen_range(1..=len);
                                self.cards.insert(idx, card.clone());
                                if self.current_deck != "Favorites" {
                                    if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                                        if !std::ptr::eq(deck_vec, &self.cards) {
                                            deck_vec.insert(idx, card);
                                        }
                                    }
                                }
                                self.current_index = 0;
                            }
                        }
                    }
                    AnimationState::Shuffling => {
                        let mut rng = rand::thread_rng();
                        let len = self.cards.len();
                        for i in (1..len).rev() {
                            let j = rng.gen_range(0..=i);
                            self.cards.swap(i, j);
                        }
                        self.current_index = 0;
                    }
                    AnimationState::None => {}
                }
                self.show_back = false;
                true
            }
            Msg::SelectDeck(name) => {
                if self.anim_state == AnimationState::None {
                    if let Some(deck_cards) = self.decks.get(&name) {
                        // Clone the deck's Vec so self.cards is independent
                        self.current_deck = name.clone();
                        self.cards = deck_cards.clone();
                        self.current_index = 0;
                        self.show_back = false;
                        self.initial_count = self.cards.len();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Msg::AddToFavorites => {
                if let Some(card) = self.cards.get(self.current_index).cloned() {
                    let fav = self.decks.get_mut("Favorites").unwrap();
                    if !fav.iter().any(|c| c.character == card.character) {
                        fav.push(card);
                    }
                }
                false
            }
            Msg::TogglePinyin(v) => {
                self.show_pinyin = v;
                true
            }
            Msg::ToggleEnglish(v) => {
                self.show_english = v;
                true
            }
            Msg::ToggleExamples(v) => {
                self.show_examples = v;
                true
            }
            Msg::ToggleExamplesPinyin(v) => {
                self.show_examples_pinyin = v;
                true
            }
            Msg::ToggleExamplesEnglish(v) => {
                self.show_examples_english = v;
                true
            }
            Msg::ToggleRadicals(v) => {
                self.show_radicals = v;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Build deck <option> tags
        let deck_options = self.decks.keys().map(|d| {
            let sel = *d == self.current_deck;
            html! {
                <option value={d.clone()} selected={sel}>{ d.clone() }</option>
            }
        });

        // Remaining / Removed
        let rem = self.cards.len();
        let removed = self.initial_count.saturating_sub(rem);

        // Current + next three cards for peeking
        let curr = self.cards.get(self.current_index);
        let nxt1 = if self.cards.len() > 1 {
            Some(self.cards.get((self.current_index + 1) % self.cards.len()).unwrap())
        } else {
            None
        };
        let nxt2 = if self.cards.len() > 2 {
            Some(self.cards.get((self.current_index + 2) % self.cards.len()).unwrap())
        } else {
            None
        };
        let nxt3 = if self.cards.len() > 3 {
            Some(self.cards.get((self.current_index + 3) % self.cards.len()).unwrap())
        } else {
            None
        };

        // Classes for the top card
        let mut inner_cls = classes!("card-inner");
        if self.show_back {
            inner_cls.push("flipped");
        }
        match self.anim_state {
            AnimationState::Removing => inner_cls.push("removing"),
            AnimationState::Replacing => inner_cls.push("replacing"),
            AnimationState::Shuffling => inner_cls.push("shuffling"),
            AnimationState::None => {}
        };

        html! {
            <div class="app-container"
                 ref={ self.container_ref.clone() }
                 tabindex="0"
                 onkeydown={ ctx.link().callback(|e: KeyboardEvent| {
                     match e.key().as_str() {
                         "ArrowLeft"  => Msg::StartRemove,
                         "ArrowRight" => Msg::StartReplace,
                         "s" | "S"    => Msg::StartShuffle,
                         _ => Msg::Flip,
                     }
                 }) }
            >
                // Controls row
                <div class="controls">
                  <div class="controls-left">
                    <label for="deck-select"><b>{ "Deck:" }</b></label>
                    <select id="deck-select"
                            onchange={ ctx.link().callback(|e: Event| {
                                let sel = e.target_unchecked_into::<HtmlSelectElement>();
                                Msg::SelectDeck(sel.value())
                            }) }>
                      { for deck_options }
                    </select>
                    <button onclick={ ctx.link().callback(|_| Msg::StartShuffle) }>
                      { "Shuffle" }
                    </button>
                  </div>
                  <div class="controls-right">
                    { format!("Remaining: {} Removed: {}", rem, removed) }
                  </div>
                </div>

                // Toggles
                <div class="toggles">
                  <label>
                    <input type="checkbox"
                           checked={ self.show_pinyin }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::TogglePinyin(chk.checked())
                           }) }/>
                    { "Pinyin" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_english }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::ToggleEnglish(chk.checked())
                           }) }/>
                    { "English" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::ToggleExamples(chk.checked())
                           }) }/>
                    { "Examples" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples_pinyin }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::ToggleExamplesPinyin(chk.checked())
                           }) }/>
                    { "Ex. Pinyin" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples_english }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::ToggleExamplesEnglish(chk.checked())
                           }) }/>
                    { "Ex. Eng." }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_radicals }
                           onchange={ ctx.link().callback(|e: Event| {
                              let chk = e.target_unchecked_into::<HtmlInputElement>();
                              Msg::ToggleRadicals(chk.checked())
                           }) }/>
                    { "Radicals" }
                  </label>
                </div>
                
                <div class="card-container">
                  // Always render the "No cards" card at the very bottom
                  <div class="pile-card pile-card-1" style="z-index:0;">
                    <div class="card-face front">
                      <div style="color:#888;">{ "No cards." }</div>
                    </div>
                  </div>
                  {
                    if let Some(c) = nxt3 {
                        html! {
                          <div class="pile-card pile-card-3">
                            <div class="card-face front">{ &c.character }</div>
                          </div>
                        }
                    } else { html!{} }
                  }
                  {
                    if let Some(c) = nxt2 {
                        html! {
                          <div class="pile-card pile-card-2">
                            <div class="card-face front">{ &c.character }</div>
                          </div>
                        }
                    } else { html!{} }
                  }
                  {
                    if let Some(c) = nxt1 {
                        html! {
                          <div class="pile-card pile-card-1">
                            <div class="card-face front">{ &c.character }</div>
                          </div>
                        }
                    } else { html!{} }
                  }

                  // Active card
                  {
                    if !self.cards.is_empty() {
                        html! {
                          <div class={ inner_cls.clone() }
                               onclick={ ctx.link().callback(|_| Msg::Flip) }
                               onanimationend={ ctx.link().callback(|_| Msg::AnimDone) }
                          >
                            // Front face
                            <div class="card-face front">
                              {
                                if let Some(card) = curr {
                                  html! { <div>{ &card.character }</div> }
                                } else {
                                  html! {}
                                }
                              }
                            </div>

                            // Back face
                            <div class="card-face back">
                              {
                                if let Some(card) = curr {
                                  html! {
                                    <div style="width:100%; height:100%; display:flex; flex-direction:column; overflow:hidden;">
                                      /* Line 1: CHARACTER + (pinyin) + radicals */
                                      <div style="font-size:2.8em; font-weight:bold; text-align:center; color:#333; margin-bottom:0.4em; display:flex; justify-content:center; align-items:center; flex-wrap:wrap;">
                                        { &card.character }
                                        {
                                          if self.show_pinyin {
                                            html! {
                                              <span style="margin-left:0.4em; font-size:1.15em; color:#555;">
                                                { "(" }{ &card.pinyin }{ ")" }
                                              </span>
                                            }
                                          } else {
                                            html! {}
                                          }
                                        }
                                        {
                                          if self.show_radicals {
                                            html! {
                                              <span style="margin-left:0.6em; font-size:1em; color:#777; display:inline-block; margin-top:0.2em;">
                                                { for card.radicals.iter().map(|r| {
                                                  html! {
                                                    <>
                                                      <span style="font-size:1.1em;">{ &r.character }</span>
                                                      <span style="font-size:0.85em;">{ "(" }{ &r.pinyin }{ "－" }{ &r.meaning }{ ")" }</span>
                                                      { " " }
                                                    </>
                                                  }
                                                })}
                                              </span>
                                            }
                                          } else {
                                            html! {}
                                          }
                                        }
                                      </div>

                                      /* Line 2: definitions */
                                      {
                                        if self.show_english {
                                          html! {
                                            <div style="text-align:center; font-size:1.2em; margin-bottom:0.6em; color:#444;">
                                              { &card.english }
                                            </div>
                                          }
                                        } else {
                                          html! {}
                                        }
                                      }

                                      /* Lines 3+: examples */
                                      {
                                        if self.show_examples {
                                          html! {
                                            <div style="flex-grow:1; overflow:hidden; margin-top:0.3em;">
                                              { for card.examples.iter().map(|ex| html! {
                                                <div style="margin-bottom:0.6em;">
                                                  <div class="example-chinese">{ &ex.chinese }</div>
                                                  { if self.show_examples_pinyin {
                                                      html! {
                                                        <div class="example-pinyin">{ &ex.pinyin }</div>
                                                      }
                                                    } else { html!{} } }
                                                  { if self.show_examples_english {
                                                      html! {
                                                        <div class="example-english">{ &ex.english }</div>
                                                      }
                                                    } else { html!{} } }
                                                </div>
                                              })}
                                            </div>
                                          }
                                        } else {
                                          html! {}
                                        }
                                      }
                                    </div>
                                  }
                                } else {
                                  html! {}
                                }
                              }
                            </div>
                          </div>
                        }
                    } else {
                        html! {}
                    }
                  }
                </div>

                // Buttons under card
                <div style="text-align:center; margin-bottom:30px;">
                  <button
                    onclick={ ctx.link().callback(|_| Msg::StartRemove) }
                    disabled={
                      self.cards.is_empty()
                      || matches!(self.anim_state, AnimationState::Removing
                                               | AnimationState::Replacing
                                               | AnimationState::Shuffling)
                    }
                  >{ "I know this – Remove" }</button>

                  <button
                    onclick={ ctx.link().callback(|_| Msg::StartReplace) }
                    disabled={
                      self.cards.is_empty()
                      || matches!(self.anim_state, AnimationState::Removing
                                               | AnimationState::Replacing
                                               | AnimationState::Shuffling)
                    }
                    style="margin-left:1em;"
                  >{ "I don’t know – Replace" }</button>

                  <button
                    onclick={ ctx.link().callback(|_| Msg::AddToFavorites) }
                    disabled={ self.cards.is_empty() || self.current_deck == "Favorites" }
                    style="margin-left:1em;"
                  >{ "Add to Favorites" }</button>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
