use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;
use std::cmp::Ordering;
use {Clock, Span};

pub struct Scheduler<T> {
    inner: BinaryHeap<Schedule<T>>,
}

impl<T> Scheduler<T> {
    pub fn new() -> Scheduler<T> {
        Scheduler {
            inner: BinaryHeap::new(),
        }
    }
    pub fn push(&mut self, schedule: ScheduleType, action: T) {
        let schedule = Schedule::new(action, schedule);
        self.inner.push(schedule);
    }
}

impl<T: Clone> Scheduler<T> {
    pub fn pop(&mut self, count: Clock) -> Vec<T> {
        let count_cmp = u64::max_value() - count;
        let mut res = vec![];
        let mut nexts = BinaryHeap::new();
        while let Some(schedule) = self.inner.peek_mut() {
            if schedule.key < count_cmp {
                break;
            }
            let schedule = PeekMut::pop(schedule);
            res.push(schedule.action.clone());
            match schedule.typ {
                ScheduleType::Span(mut s) => {
                    if s.length() <= 1 {
                        continue;
                    }
                    s.start += 1;
                    let next = Schedule::new(schedule.action, ScheduleType::Span(s));
                    nexts.push(next);
                }
                ScheduleType::RepeatedOnce { span: s, next: _ } => {
                    let typ = ScheduleType::RepeatedOnce {
                        span: s,
                        next: count + s,
                    };
                    let next = Schedule::new(schedule.action, typ);
                    nexts.push(next);
                }
                ScheduleType::RepeatedSpan {
                    span: s,
                    next: mut n,
                    exec_span: e,
                } => {
                    let next_span = if n.length() <= 1 {
                        let end = count + s;
                        let start = end + 1 - e;
                        Span::new(start, end)
                    } else {
                        n.start += 1;
                        n
                    };
                    let typ = ScheduleType::RepeatedSpan {
                        span: s,
                        next: next_span,
                        exec_span: e,
                    };
                    if next_span.start > count {
                        let next = Schedule::new(schedule.action, typ);
                        nexts.push(next);
                    }
                }
                _ => {}
            }
        }
        self.inner.append(&mut nexts);
        res
    }
}

struct Schedule<T> {
    action: T,
    typ: ScheduleType,
    key: Clock,
}

impl<T> Schedule<T> {
    fn new(action: T, typ: ScheduleType) -> Schedule<T> {
        let key = u64::max_value() ^ typ.start();
        Schedule {
            action: action,
            typ: typ,
            key: key,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleType {
    Once(Clock),
    Span(Span),
    RepeatedOnce {
        span: Clock,
        next: Clock,
    },
    RepeatedSpan {
        // start to start
        span: Clock,
        next: Span,
        exec_span: Clock,
    },
}

impl ScheduleType {
    fn start(&self) -> Clock {
        match *self {
            ScheduleType::Once(c) => c,
            ScheduleType::Span(s) => s.start,
            ScheduleType::RepeatedOnce { span: _, next: c } => c,
            ScheduleType::RepeatedSpan {
                span: _,
                next: s,
                exec_span: _,
            } => s.start,
        }
    }
    fn once(start: Clock) -> ScheduleType {
        ScheduleType::Once(start)
    }
    fn span(start: Clock, span: Clock) -> ScheduleType {
        ScheduleType::Span(Span::new(start, start + span - 1))
    }
    fn repeated_once(start: Clock, span: Clock) -> ScheduleType {
        ScheduleType::RepeatedOnce {
            span: span,
            next: start,
        }
    }
    fn repeated_span(start: Clock, span: Clock, exec_span: Clock) -> ScheduleType {
        let next = Span::new(start, start + exec_span - 1);
        ScheduleType::RepeatedSpan {
            span: span,
            next: next,
            exec_span: exec_span,
        }
    }
}

impl<T> Ord for Schedule<T> {
    fn cmp(&self, other: &Schedule<T>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<T> PartialOrd for Schedule<T> {
    fn partial_cmp(&self, other: &Schedule<T>) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

impl<T> PartialEq for Schedule<T> {
    fn eq(&self, other: &Schedule<T>) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for Schedule<T> {}

#[cfg(test)]
mod schedule_test {
    use super::*;
    #[test]
    fn once() {
        let mut scheduler = Scheduler::<String>::new();
        let once = ScheduleType::once(5);
        scheduler.push(once, "@_@".to_string());
        (0..10).for_each(|i| {
            let v = scheduler.pop(i);
            if i != 5 {
                assert!(v.is_empty());
            } else {
                assert!(v.len() == 1);
                assert_eq!(v.get(0).unwrap(), "@_@");
            }
        });
    }
    #[test]
    fn span() {
        let mut scheduler = Scheduler::<String>::new();
        let span = ScheduleType::span(5, 5);
        scheduler.push(span, "@_@".to_string());
        (0..15).for_each(|i| {
            let v = scheduler.pop(i);
            if 5 <= i && i < 10 {
                assert!(v.len() == 1);
                assert_eq!(v.get(0).unwrap(), "@_@");
            } else {
                assert!(v.is_empty());
            }
        });
    }
    #[test]
    fn repeated_once() {
        let mut scheduler = Scheduler::<String>::new();
        let span = ScheduleType::repeated_once(3, 3);
        scheduler.push(span, "@_@".to_string());
        (0..15).for_each(|i| {
            let v = scheduler.pop(i);
            if i % 3 == 0 && i != 0 {
                assert!(v.len() == 1);
                assert_eq!(v.get(0).unwrap(), "@_@");
            } else {
                assert!(v.is_empty());
            }
        });
    }
    #[test]
    fn repeated_span() {
        let mut scheduler = Scheduler::<String>::new();
        let span = ScheduleType::repeated_span(3, 3, 2);
        scheduler.push(span, "@_@".to_string());
        (0..15).for_each(|i| {
            let v = scheduler.pop(i);
            if i % 3 != 2 && i >= 3 {
                assert!(v.len() == 1);
                assert_eq!(v.get(0).unwrap(), "@_@");
            } else {
                assert!(v.is_empty(), "i: {}", i);
            }
        });
    }
}
