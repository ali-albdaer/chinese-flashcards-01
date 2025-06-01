use std::collections::HashMap;
use rand::Rng;
use serde::Deserialize;
use yew::prelude::*;
use web_sys::{HtmlSelectElement, HtmlInputElement, HtmlElement};

//───────────────────────────────────────────────────────────────────────────────
// 1. Data structures (deserialize from JSON)
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
// 2. Animation state enum
//───────────────────────────────────────────────────────────────────────────────

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
    // All decks from JSON
    decks: DecksMap,
    current_deck: String,
    cards: Vec<Card>,
    current_index: usize,
    show_back: bool,
    anim_state: AnimationState,

    // Toggles for back‐side fields
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
        // Include JSON at compile time
        let raw_json = include_str!("cards.json");
        let mut decks: DecksMap = serde_json::from_str(raw_json)
            .expect("cards.json must be valid JSON");

        // Ensure "Favorites" exists
        if !decks.contains_key("Favorites") {
            decks.insert("Favorites".to_string(), Vec::new());
        }

        // Default deck: HSK3
        let current_deck = "HSK3".to_string();
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
            show_radicals: false, // disabled by default

            initial_count,
            container_ref: NodeRef::default(),
            focus_set: false,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        // Focus the container on first render so arrow keys work
        if first_render && !self.focus_set {
            if let Some(elem) = self.container_ref.cast::<HtmlElement>() {
                let _ = elem.focus();
                self.focus_set = true;
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Flip => {
                if let AnimationState::None = self.anim_state {
                    self.show_back = !self.show_back;
                    true
                } else {
                    false
                }
            }

            Msg::StartRemove => {
                if let AnimationState::None = self.anim_state {
                    self.anim_state = AnimationState::Removing;
                    self.show_back = false;
                    true
                } else {
                    false
                }
            }

            Msg::StartReplace => {
                if let AnimationState::None = self.anim_state {
                    self.anim_state = AnimationState::Replacing;
                    self.show_back = false;
                    true
                } else {
                    false
                }
            }

            Msg::StartShuffle => {
                if let AnimationState::None = self.anim_state {
                    self.anim_state = AnimationState::Shuffling;
                    true
                } else {
                    false
                }
            }

            Msg::AnimDone => {
                // After any animation ends, finalize the action
                match std::mem::replace(&mut self.anim_state, AnimationState::None) {
                    AnimationState::Removing => {
                        if !self.cards.is_empty() {
                            let _removed = self.cards.remove(self.current_index);
                            if self.current_deck != "Favorites" {
                                if let Some(deck_vec) = self.decks.get_mut(&self.current_deck) {
                                    if self.current_index < deck_vec.len() {
                                        deck_vec.remove(self.current_index);
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
                                    if self.current_index < deck_vec.len() {
                                        deck_vec.remove(self.current_index);
                                    }
                                }
                            }
                            let mut rng = rand::thread_rng();
                            let len = self.cards.len();
                            if len == 0 {
                                self.cards.push(card.clone());
                                if self.current_deck != "Favorites" {
                                    self.decks.get_mut(&self.current_deck).unwrap().push(card);
                                }
                                self.current_index = 0;
                            } else {
                                // Insert at random index ≥ 1 so that top card always changes
                                let idx = rng.gen_range(1..=len);
                                self.cards.insert(idx, card.clone());
                                if self.current_deck != "Favorites" {
                                    self.decks.get_mut(&self.current_deck).unwrap().insert(idx, card);
                                }
                                self.current_index = 0;
                            }
                        }
                    }
                    AnimationState::Shuffling => {
                        // Fisher–Yates shuffle
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

            Msg::SelectDeck(deck_name) => {
                if let AnimationState::None = self.anim_state {
                    if let Some(deck_cards) = self.decks.get(&deck_name) {
                        self.current_deck = deck_name.clone();
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
                    let fav_deck = self.decks.get_mut("Favorites").unwrap();
                    let already = fav_deck.iter().any(|c| c.character == card.character);
                    if !already {
                        fav_deck.push(card);
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
        let deck_options = self.decks.keys().map(|deck_name| {
            let selected = *deck_name == self.current_deck;
            html! {
                <option value={deck_name.clone()} selected={selected}>{ deck_name.clone() }</option>
            }
        });

        // Remaining & Removed counts
        let remaining = self.cards.len();
        let removed = self.initial_count.saturating_sub(remaining);

        // Current card & next three cards (for the multi‐level pile)
        let card_opt = self.cards.get(self.current_index);
        let next1 = if self.cards.len() > 1 {
            Some(self.cards.get((self.current_index + 1) % self.cards.len()).unwrap())
        } else {
            None
        };
        let next2 = if self.cards.len() > 2 {
            Some(self.cards.get((self.current_index + 2) % self.cards.len()).unwrap())
        } else {
            None
        };
        let next3 = if self.cards.len() > 3 {
            Some(self.cards.get((self.current_index + 3) % self.cards.len()).unwrap())
        } else {
            None
        };

        // CSS classes for the active card
        let mut inner_classes = classes!("card-inner");
        if self.show_back {
            inner_classes.push("flipped");
        }
        match self.anim_state {
            AnimationState::Removing => inner_classes.push("removing"),
            AnimationState::Replacing => inner_classes.push("replacing"),
            AnimationState::Shuffling => inner_classes.push("shuffling"),
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
                //──────────────────────────────────────────────────────────────────────
                // Controls row: Deck, Shuffle, Counter
                <div class="controls">
                  <div class="controls-left">
                    <label for="deck-select"><b>{ "Deck:" }</b></label>
                    <select id="deck-select"
                        onchange={ ctx.link().callback(|e: Event| {
                            let sel = e.target_unchecked_into::<HtmlSelectElement>();
                            Msg::SelectDeck(sel.value())
                        }) }
                    >
                      { for deck_options }
                    </select>
                    <button onclick={ ctx.link().callback(|_| Msg::StartShuffle) }>
                      { "Shuffle" }
                    </button>
                  </div>
                  <div class="controls-right">
                    { format!("Remaining: {} Removed: {}", remaining, removed) }
                  </div>
                </div>

                //──────────────────────────────────────────────────────────────────────
                // Toggle checkboxes
                <div class="toggles">
                  <label>
                    <input type="checkbox"
                           checked={ self.show_pinyin }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::TogglePinyin(chk.checked())
                           }) } />
                    { "Pinyin" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_english }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::ToggleEnglish(chk.checked())
                           }) } />
                    { "English" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::ToggleExamples(chk.checked())
                           }) } />
                    { "Examples" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples_pinyin }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::ToggleExamplesPinyin(chk.checked())
                           }) } />
                    { "Ex. Pinyin" }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_examples_english }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::ToggleExamplesEnglish(chk.checked())
                           }) } />
                    { "Ex. Eng." }
                  </label>
                  <label>
                    <input type="checkbox"
                           checked={ self.show_radicals }
                           onchange={ ctx.link().callback(|e: Event| {
                             let chk = e.target_unchecked_into::<HtmlInputElement>();
                             Msg::ToggleRadicals(chk.checked())
                           }) } />
                    { "Radicals" }
                  </label>
                </div>

                //──────────────────────────────────────────────────────────────────────
                // Card “pile” + active card
                <div class="card-container">
                  {
                    // Show up to three peek cards behind
                    if let Some(c) = next3 {
                      html! {
                        <div class="pile-card-3" style={
                          // When animating top card, ramp opacity -> 1
                          if !matches!(self.anim_state, AnimationState::None) {
                            "opacity:1;"
                          } else {
                            ""
                          }
                        }>
                          <div class="card-face front">{ &c.character }</div>
                        </div>
                      }
                    } else {
                      html! {}
                    }
                  }
                  {
                    if let Some(c) = next2 {
                      html! {
                        <div class="pile-card-2" style={
                          if !matches!(self.anim_state, AnimationState::None){
                            "opacity:1;"
                          } else { "" }
                        }>
                          <div class="card-face front">{ &c.character }</div>
                        </div>
                      }
                    } else {
                      html! {}
                    }
                  }
                  {
                    if let Some(c) = next1 {
                      html! {
                        <div class="pile-card-1" style={
                          if !matches!(self.anim_state, AnimationState::None) {
                            "opacity:1;"
                          } else { "" }
                        }>
                          <div class="card-face front">{ &c.character }</div>
                        </div>
                      }
                    } else {
                      html! {}
                    }
                  }

                  // The active, flipping/animating card on top
                  <div class={ inner_classes.clone() }
                       onclick={ ctx.link().callback(|_| Msg::Flip) }
                       onanimationend={ ctx.link().callback(|_| Msg::AnimDone) }
                  >
                    // Front face: large character
                    <div class="card-face front">
                      {
                        if let Some(card) = card_opt {
                          html! { <div>{ &card.character }</div> }
                        } else {
                          html! { <div style="color:#888;">{ "No cards." }</div> }
                        }
                      }
                    </div>

                    // Back face: fixed layout
                    <div class="card-face back">
                      {
                        if let Some(card) = card_opt {
                          html! {
                            <div style="width:100%; height:100%; display:flex; flex-direction:column; overflow:hidden;">
                              /*─── Line 1 (center): CHARACTER (pinyin) {Radicals} */
                              <div style="font-size:2.8em; font-weight:bold; text-align:center; color:#333; margin-bottom:0.4em; display:flex; justify-content:center; align-items:center; flex-wrap:wrap;">
                                { &card.character }
                                <span style="margin-left:0.4em; font-size:1.15em; color:#555;">
                                  { "(" }{ &card.pinyin }{ ")" }
                                </span>
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

                              /*─── Line 2 (definitions) */
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

                              /*─── Lines 3+: example sentences */
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
                          html! { <div style="color:#888;">{ "No cards in this deck." }</div> }
                        }
                      }
                    </div>
                  </div>
                </div>

                /*──────────────────────────────────────────────────────────────────────*/
                // Buttons
                <div style="margin-top:1em; text-align:center;">
                  <button
                    onclick={ ctx.link().callback(|_| Msg::StartRemove) }
                    disabled={
                      self.cards.is_empty()
                      || matches!(self.anim_state, AnimationState::Removing
                                               | AnimationState::Replacing
                                               | AnimationState::Shuffling)
                    }
                  >{ "I know this (Remove)" }</button>

                  <button
                    onclick={ ctx.link().callback(|_| Msg::StartReplace) }
                    disabled={
                      self.cards.is_empty()
                      || matches!(self.anim_state, AnimationState::Removing
                                               | AnimationState::Replacing
                                               | AnimationState::Shuffling)
                    }
                    style="margin-left:1em;"
                  >{ "I don't know (Replace)" }</button>

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
    // Yew 0.20+ uses Renderer
    yew::Renderer::<App>::new().render();
}
