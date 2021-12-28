use yew::prelude::*;

pub enum HomeMsg {
    AddOne,
}
type Msg = HomeMsg;

pub struct Home{
    value: i64,
}


impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                log::info!("Adding 1");
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_addone_click = ctx.link().callback(|_| Msg::AddOne);
        html! {
            <div class="tile is-ancestor is-vertical">
                <div class="tile is-child hero">
                    <div class="hero-body container pb-0">
                        <h1 class="title is-1">{ "Welcome..." }</h1>
                        <h2 class="subtitle">{ "...to the best yew content" }</h2>
                    </div>
                </div>

                <button onclick={on_addone_click}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}
