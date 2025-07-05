use std::collections::HashMap;
use rand::Rng;
use serde::Deserialize;
use yew::prelude::*;
use web_sys::{HtmlSelectElement, HtmlInputElement, HtmlElement};


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

#[derive(PartialEq, Eq)]
enum AnimationState {
    None,
    Removing,
    Replacing,
    Shuffling,
}

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
        let hsk1: Vec<Card> = serde_json::from_str(include_str!("decks/HSK1.json"))
            .expect("Failed to parse decks/HSK1.json");
        let hsk2: Vec<Card> = serde_json::from_str(include_str!("decks/HSK2.json"))
            .expect("Failed to parse decks/HSK2.json");
        let hsk3: Vec<Card> = serde_json::from_str(include_str!("decks/HSK3.json"))
            .expect("Failed to parse decks/HSK3.json");
        let hsk4: Vec<Card> = serde_json::from_str(include_str!("decks/HSK4.json"))
            .expect("Failed to parse decks/HSK4.json");
        let chn203: Vec<Card> = serde_json::from_str(include_str!("decks/CHN203.json"))
            .expect("Failed to parse decks/CHN203.json");
        let collection: Vec<Card> = serde_json::from_str(include_str!("decks/COLLECTION.json"))
            .expect("Failed to parse decks/COLLECTION.json");

        // Insert decks into the map
        decks.insert("HSK1".into(), hsk1.clone());
        decks.insert("HSK2 (INCMPLT)".into(), hsk2.clone());
        decks.insert("HSK3 (INCMPLT)".into(), hsk3.clone());
        decks.insert("HSK4 (INCMPLT)".into(), hsk4.clone());
        decks.insert("In Class".into(), chn203.clone());
        decks.insert("Collection".into(), collection.clone());

        // Default to CHN203 deck
        let current_deck = "In Class".into();
        let cards = decks.get(&current_deck).unwrap().clone();
        let initial_count = cards.len();

        // Such description cards can be added:
        //   {
        //   "character": "HSK3级词汇",
        //   "pinyin": "HSK3 jí cíhuì",
        //   "english": "HSK Level 3 Vocabulary",
        //   "examples": [
        //     { "chinese": "HSK3级词汇包括常用的汉字和词语。", "pinyin": "HSK3 jí cíhuì bāokuò chángyòng de hànzì hé cíyǔ。", "english": "HSK Level 3 vocabulary includes commonly used Chinese characters and words." },
        //     { "chinese": "首先翻开卡片，看看它们的含义、例句等。", "pinyin": "Shǒuxiān fānkāi kǎpiàn, kàn kàn tāmen de hányì, lìjù děng。", "english": "First, open the cards and look at their meanings, example sentences, etc." },
        //     { "chinese": "祝你学业顺利！", "pinyin": "Zhù nǐ xuéyè shùnlì！", "english": "Good luck in your studies!" }
        //   ],
        //   "radicals": [
        //     { "character": "字", "pinyin": "zì", "meaning": "character" },
        //     { "character": "词", "pinyin": "cí", "meaning": "word" },
        //     { "character": "汇", "pinyin": "huì", "meaning": "collection" }
        //   ]
        // },

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
            show_radicals: true,

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
        // Build deck <option> tags, sorted alphabetically
        let mut deck_names: Vec<_> = self.decks.keys().cloned().collect();
        deck_names.sort();
        let deck_options = deck_names.iter().map(|d| {
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
                         " " | "Spacebar" | "ArrowUp" | "ArrowDown" => Msg::Flip,
                         "s" | "S"    => Msg::StartShuffle,
                         _ => Msg::AnimDone,
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
                        let char_len = c.character.chars().count();
                        let char_font_size = match char_len {
                            1 => "10em",
                            2 => "10em",
                            3 => "9em",
                            4 => "7em",
                            5 => "5em",
                            _ => "4em",
                        };
                        html! {
                          <div class="pile-card pile-card-3">
                            <div class="card-face front">
                              <div style={format!("font-size:{};", char_font_size)}>{ &c.character }</div>
                            </div>
                          </div>
                        }
                    } else { html!{} }
                  }
                  {
                    if let Some(c) = nxt2 {
                        let char_len = c.character.chars().count();
                        let char_font_size = match char_len {
                            1 => "10em",
                            2 => "10em",
                            3 => "9em",
                            4 => "7em",
                            5 => "5em",
                            _ => "4em",
                        };
                        html! {
                          <div class="pile-card pile-card-2">
                            <div class="card-face front">
                              <div style={format!("font-size:{};", char_font_size)}>{ &c.character }</div>
                            </div>
                          </div>
                        }
                    } else { html!{} }
                  }
                  {
                    if let Some(c) = nxt1 {
                        let char_len = c.character.chars().count();
                        let char_font_size = match char_len {
                            1 => "10em",
                            2 => "10em",
                            3 => "9em",
                            4 => "7em",
                            5 => "5em",
                            _ => "4em",
                        };
                        html! {
                          <div class="pile-card pile-card-1">
                            <div class="card-face front">
                              <div style={format!("font-size:{};", char_font_size)}>{ &c.character }</div>
                            </div>
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
                                  let char_len = card.character.chars().count();
                                  let char_font_size = match char_len {
                                    1 => "10em",
                                    2 => "10em",
                                    3 => "9em",
                                    4 => "7em",
                                    5 => "5em",
                                    _ => "4em",
                                  };
                                  html! { <div style={format!("font-size:{};", char_font_size)}>{ &card.character }</div> }
                                } else {
                                  html! {}
                                }
                              }
                            </div>

                            // Back face
                            <div class="card-face back" style="position:relative;">
                              {
                                if let Some(card) = curr {
                                  let char_len = card.character.chars().count();
                                  let char_font_size = if char_len >= 4 {
                                    "1.7em"
                                  } else if char_len == 3 {
                                    "2.1em"
                                  } else {
                                    "2.8em"
                                  };
                                  html! {
                                    <div style="width:100%; height:100%; display:flex; flex-direction:column; overflow:hidden; position:relative;">
                                      // Line 1: CHARACTER + (pinyin)
                                      <div style={format!(
                                        "font-size:{}; font-weight:bold; text-align:center; color:#333; margin-bottom:0.1em; display:flex; justify-content:center; align-items:center; flex-wrap:wrap;",
                                        char_font_size
                                      )}>
                                        { &card.character }
                                        {
                                          if self.show_pinyin {
                                            html! {
                                              <span style="margin-left:0.4em; font-size:1em; color:#555;">
                                                { "(" }{ &card.pinyin }{ ")" }
                                              </span>
                                            }
                                          } else {
                                            html! {}
                                          }
                                        }
                                      </div>
                                      // Line 2: definitions
                                      {
                                        if self.show_english {
                                          html! {
                                            <div style="text-align:center; font-size:1.2em; margin-bottom:0.1em; color:#444;">
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
                                            <div style="flex-grow:1; overflow:hidden; margin-top:0.1em;">
                                              { for card.examples.iter().map(|ex| html! {
                                                <div style="margin-bottom:0.1em;">
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

                                      // Radicals in bottom right corner
                                      {
                                        if self.show_radicals && !card.radicals.is_empty() {
                                          html! {
                                            <div class="radicals-corner" style="
                                              display: flex;
                                              flex-direction: column-reverse;
                                              align-items: flex-end;
                                              gap: 0.15em;
                                            ">
                                              { for card.radicals.iter().rev().map(|r| html! {
                                                <span style="white-space:nowrap;">
                                                  <span style="font-size:1.1em;">{ &r.character }</span>
                                                  <span style="font-size:0.85em; margin-left:0.15em;">{ "(" }{ &r.pinyin }{ "－" }{ &r.meaning }{ ")" }</span>
                                                </span>
                                              }) }
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
                <div class="card-buttons-row">
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
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
