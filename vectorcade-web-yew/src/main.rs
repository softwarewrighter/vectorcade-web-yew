use yew::prelude::*;
use vectorcade_games::all_games;
use vectorcade_shared::draw::DrawCmd;

#[function_component(App)]
fn app() -> Html {
    let games = use_memo(|_| all_games(), ());
    let selected = use_state(|| 0usize);

    let on_change = {
        let selected = selected.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            selected.set(target.value().parse::<usize>().unwrap_or(0));
        })
    };

    html! {
        <div style="position: relative; height: 100%;">
          <div class="hud">{ "VectorCade (skeleton)" }</div>
          <div class="panel">
            <select onchange={on_change}>
              { for games.iter().enumerate().map(|(i,g)| html!{
                  <option value={i.to_string()} selected={*selected == i}>
                    { g.metadata().name }
                  </option>
              })}
            </select>
            <button onclick={{
                let selected = selected.clone();
                Callback::from(move |_| { selected.set(*selected); })
            }}>{ "Reset (stub)" }</button>
          </div>

          // Canvas + render loop wiring is Agent work.
          <div style="position: absolute; inset: 0;">
            <canvas id="vectorcade-canvas"></canvas>
          </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
