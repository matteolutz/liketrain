use chumsky::prelude::*;

mod section;
pub use section::*;

pub mod eval;

use crate::{SectionId, SwitchState};

#[cfg(test)]
mod tests;

pub fn parser<'src>()
-> impl Parser<'src, &'src str, Vec<TrackDefinition<'src>>, extra::Err<Rich<'src, char>>> {
    let comment = just('#')
        .padded()
        .ignore_then(any().filter(|c| *c != '\n').repeated())
        .ignored();

    let ws = choice((text::whitespace(), comment)).ignored();

    let ident = text::ident().padded();

    let section_id = just("S")
        .ignore_then(text::int(10).from_str().unwrapped().map(SectionId::new))
        .padded();

    let switch_state = choice((
        just("left").to(SwitchState::Left),
        just("right").to(SwitchState::Right),
    ))
    .padded();

    // direct(S2)
    let direct = just("direct")
        .padded()
        .ignore_then(section_id.delimited_by(just('('), just(')')))
        .map(|to| ConnectionExpr::Direct { to });

    // switch(A)
    let switch = just("switch")
        .padded()
        .ignore_then(ident.delimited_by(just('('), just(')')))
        .map(|switch_name| ConnectionExpr::Switch { switch_name });

    //  switch(A, left)
    let switch_with_sate = just("switch")
        .padded()
        .ignore_then(
            ident
                .then_ignore(just(',').padded())
                .then(switch_state)
                .delimited_by(just('('), just(')')),
        )
        .map(|(switch_name, state)| SwitchWithState { switch_name, state });

    // back(A, left)
    let back = just("back")
        .padded()
        .ignore_then(
            ident
                .then_ignore(just(',').padded())
                .then(switch_state)
                .delimited_by(just('('), just(')')),
        )
        .map(|(switch_name, required_state)| ConnectionExpr::SwitchBack {
            switch_name,
            required_state,
        });

    let none = just("none").padded().to(ConnectionExpr::None);

    let connection_expr = none.or(direct).or(switch).or(back);

    let section_def = section_id
        .then_ignore(just(':').padded())
        .then_ignore(just("->").padded())
        .then(connection_expr.clone())
        .then_ignore(just('|').padded())
        .then_ignore(just("<-").padded())
        .then(connection_expr)
        .map(|((section_id, forward), backward)| SectionDef {
            section_id,
            forward,
            backward,
        });

    let switch_def = switch_with_sate
        .padded()
        .then_ignore(just("->").padded())
        .then(switch_with_sate)
        .map(|(from, to)| SwitchConnection { from, to });

    let def = section_def
        .map(TrackDefinition::Section)
        .or(switch_def.map(TrackDefinition::Switch));

    def.padded_by(ws).repeated().collect().then_ignore(end())
}
