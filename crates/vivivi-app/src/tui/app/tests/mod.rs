use super::*;

#[allow(dead_code)]
pub(super) fn vi_cmd_bang(vi: &mut ViEngine) -> Option<vivivi_vi::ViCommand> {
    vi.feed_char('!')
}

mod basic;
mod ex;
mod features;
