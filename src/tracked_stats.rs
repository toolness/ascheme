use std::collections::HashMap;

use crate::string_interner::InternedString;

#[derive(Default)]
pub struct TrackedCallableStats {
    calls: usize,
    tail_calls: usize,
}

#[derive(Default)]
pub struct TrackedStats {
    max_call_stack_depth: usize,
    callable_calls: HashMap<InternedString, TrackedCallableStats>,
}

impl TrackedStats {
    pub fn update_call_stack_depth(&mut self, new_depth: usize) {
        if new_depth > self.max_call_stack_depth {
            self.max_call_stack_depth = new_depth;
        }
    }

    pub fn track_tail_call(&mut self, name: Option<&InternedString>) {
        if let Some(name) = name {
            let stats = self.callable_calls.entry(name.clone()).or_default();
            stats.tail_calls += 1;
        }
    }

    pub fn track_call(&mut self, name: Option<&InternedString>) {
        if let Some(name) = name {
            let stats = self.callable_calls.entry(name.clone()).or_default();
            stats.calls += 1;
        }
    }

    pub fn as_table(&self) -> String {
        let mut lines = vec![];
        lines.push(format!("{:40} {:8} {:12}", "Name", "Calls", "Tail calls"));
        lines.push("-".repeat(60));
        let mut table_lines = self
            .callable_calls
            .iter()
            .map(|(name, stats)| {
                format!(
                    "{:40} {:8} {:12}",
                    name.to_string(),
                    stats.calls.to_string(),
                    stats.tail_calls.to_string()
                )
            })
            .collect::<Vec<String>>();
        table_lines.sort();
        lines.extend(table_lines);
        lines.push(format!(
            "\nMaximum call stack depth: {}",
            self.max_call_stack_depth
        ));
        lines.join("\n")
    }
}
