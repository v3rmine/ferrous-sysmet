use maud::{html, Markup, DOCTYPE};

use crate::components::Head;

pub fn Base(children: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (Head("Ferrous Sysmet"))
            body {
                main .container { (children) }
            }
        }
    }
}
