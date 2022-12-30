use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::source::{CodePoint, Source};

#[derive(Clone, PartialEq)]
pub(crate) struct Span {
    pub(crate) source: Rc<Source>,
    pub(crate) start: usize,
    pub(crate) end: usize
}

impl Span {
    pub(crate) fn single(p: CodePoint) -> Self{
        Self {
            source: p.0,
            start: p.1,
            end: p.1
        }
    }

    pub(crate) fn from_points(a: CodePoint, b: CodePoint) -> Self{
        assert!(Rc::ptr_eq(&a.0, &b.0), "CodePoints should be of same Source");
        Self {
            source: a.0.clone(),
            start: usize::min(a.1, b.1),
            end: usize::max(a.1, b.1)
        }
    }

    pub(crate) fn bounds(&self) -> (CodePoint, CodePoint) {
        (CodePoint(self.source.clone(), self.start),
         CodePoint(self.source.clone(), self.end))
    }

    pub(crate) fn start(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.start)
    }

    pub(crate) fn end(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.end)
    }

    pub(crate) fn extend(&mut self, p: CodePoint) {
        assert!(Rc::ptr_eq(&self.source, &p.0), "CodePoint should be of same Source as Span");
        self.start = usize::min(self.start, p.1);
        self.end = usize::max(self.end, p.1);
    }

    pub(crate) fn render_span_code(&self, line_pad: usize) -> String {
        let (sl, sp) = self.start().pos();
        let (el, ep) = self.end().pos();
        let lines_split = &self.source.source.split("\n").collect::<Vec<&str>>();
        let mut render = vec![];
        for i in usize::max(sl.saturating_sub(line_pad), 1)..=usize::min(el+line_pad, lines_split.len()) {
            render.push(format!("{i:3} | {}", lines_split[i-1]));
            if i == sl && i == el {
                render.push(format!("    | {}{}", " ".repeat(sp), "^".repeat(ep - sp + 1)));
            }
            else if i == sl {
                render.push(format!("    | {}{}", " ".repeat(sp), "^".repeat(lines_split[i-1].len() - sp + 1)));
            }
            else if i == el {
                render.push(format!("    | {}{}", "^".repeat(ep + 1), " ".repeat(lines_split[i-1].len() - ep)));
            }
            else if i > sl && i < el {
                render.push(format!("    | {}", "^".repeat(lines_split[i-1].len())));
            }
        }
        render.join("\n")
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            let (l, p) = loc.start().pos();
            write!("{}:{}", l, p)
        } else {
            let (sl, sp) = self.start().pos();
            let (el, ep) = self.end().pos();
            write!("{}:{}..{}:{}", sl, sp, el, ep)
        }
    }
}