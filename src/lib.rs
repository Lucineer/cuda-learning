/*!
# cuda-learning

Experience-based learning for agents.

Adaptation changes parameters. Learning changes behavior.
The difference: adaptation is reactive (feedback → adjust), learning is
constructive (experience → knowledge → better decisions in new situations).

This crate provides:
- Experience recording with context
- Credit assignment — which actions caused which outcomes
- Generalization — apply lessons from one context to similar contexts
- Curriculum learning — easier problems first
- Forgetting curve — old lessons fade
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learning experience
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub id: u64,
    pub context: Vec<String>,       // situation tags
    pub actions: Vec<String>,       // what was done
    pub outcome: f64,               // -1 to 1
    pub reward: f64,                // scalar reward
    pub timestamp: u64,
    pub lesson: String,             // extracted lesson
    pub generalizable: bool,        // can this apply elsewhere?
    pub context_hash: u64,          // for similarity matching
}

impl Experience {
    pub fn new(id: u64) -> Self {
        Experience { id, context: vec![], actions: vec![], outcome: 0.0, reward: 0.0, timestamp: now(), lesson: String::new(), generalizable: false, context_hash: 0 }
    }

    pub fn with_context(mut self, tags: Vec<&str>) -> Self {
        self.context = tags.iter().map(|s| s.to_string()).collect();
        self.context_hash = simple_hash(&self.context.join(","));
        self
    }

    pub fn with_actions(mut self, actions: Vec<&str>) -> Self {
        self.actions = actions.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_outcome(mut self, outcome: f64) -> Self {
        self.outcome = outcome.clamp(-1.0, 1.0);
        self.reward = outcome;
        self
    }
}

/// A lesson extracted from experience
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lesson {
    pub id: u64,
    pub rule: String,               // "IF context THEN action"
    pub confidence: f64,            // how reliable
    pub success_count: u32,
    pub failure_count: u32,
    pub contexts: Vec<String>,      // where this applies
    pub generalization_score: f64,  // how broadly applicable
    pub created: u64,
    pub last_used: u64,
}

impl Lesson {
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 { return 0.5; }
        (self.success_count as f64 + 1.0) / (total as f64 + 2.0) // Laplace
    }

    pub fn apply_confidence(&self, context: &[String]) -> f64 {
        let matches: f64 = self.contexts.iter().filter(|c| context.contains(c)).count() as f64;
        let context_factor = if self.contexts.is_empty() { 0.5 } else { matches / self.contexts.len() as f64 };
        self.confidence * self.success_rate() * context_factor
    }
}

/// Credit assignment — which actions contributed to outcome
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreditAssignment {
    pub action_credits: HashMap<String, f64>,  // action -> contribution score
    pub total_outcome: f64,
}

impl CreditAssignment {
    /// Simple temporal credit: assign more credit to later actions (closer to outcome)
    pub fn temporal_credit(actions: &[String], outcome: f64) -> Self {
        let n = actions.len();
        let mut credits = HashMap::new();
        for (i, action) in actions.iter().enumerate() {
            // Later actions get more credit (simplification of eligibility traces)
            let weight = (i + 1) as f64 / n as f64;
            credits.insert(action.clone(), outcome * weight);
        }
        CreditAssignment { action_credits: credits, total_outcome: outcome }
    }

    /// Equal credit: all actions equally responsible
    pub fn equal_credit(actions: &[String], outcome: f64) -> Self {
        let n = actions.len().max(1);
        let per_action = outcome / n as f64;
        let credits: HashMap<String, f64> = actions.iter().map(|a| (a.clone(), per_action)).collect();
        CreditAssignment { action_credits: credits, total_outcome: outcome }
    }
}

/// Curriculum — ordered learning tasks from easy to hard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Curriculum {
    pub stages: Vec<CurriculumStage>,
    pub current_stage: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumStage {
    pub name: String,
    pub difficulty: f64,
    pub required_success_rate: f64,
    pub min_experiences: u32,
    pub experiences: u32,
    pub successes: u32,
}

impl Curriculum {
    pub fn new() -> Self { Curriculum { stages: vec![], current_stage: 0 } }

    pub fn add_stage(&mut self, name: &str, difficulty: f64, required_sr: f64, min_exp: u32) {
        self.stages.push(CurriculumStage { name: name.to_string(), difficulty, required_success_rate: required_sr, min_experiences: min_exp, experiences: 0, successes: 0 });
    }

    pub fn add_default(&mut self) {
        self.add_stage("exploration", 0.2, 0.3, 10);
        self.add_stage("basic_skills", 0.4, 0.5, 20);
        self.add_stage("intermediate", 0.6, 0.6, 30);
        self.add_stage("advanced", 0.8, 0.7, 50);
    }

    /// Record outcome, check for stage advancement
    pub fn record(&mut self, success: bool) -> Option<String> {
        if self.current_stage >= self.stages.len() { return None; }
        let stage = &mut self.stages[self.current_stage];
        stage.experiences += 1;
        if success { stage.successes += 1; }

        let sr = stage.successes as f64 / stage.experiences as f64;
        if sr >= stage.required_success_rate && stage.experiences >= stage.min_experiences {
            let completed = stage.name.clone();
            self.current_stage += 1;
            return Some(format!("Stage '{}' completed! Advancing to stage {}", completed,
                self.stages.get(self.current_stage).map(|s| s.name.as_str()).unwrap_or("graduated")));
        }
        None
    }

    pub fn current_difficulty(&self) -> f64 {
        self.stages.get(self.current_stage).map(|s| s.difficulty).unwrap_or(1.0)
    }
}

/// The learning engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningEngine {
    pub experiences: Vec<Experience>,
    pub lessons: HashMap<String, Lesson>,
    pub curriculum: Curriculum,
    pub next_id: u64,
    pub next_lesson_id: u64,
    pub generalization_threshold: f64,
    pub max_experiences: usize,
    pub forgetting_half_life_ms: u64,
}

impl LearningEngine {
    pub fn new() -> Self {
        let mut engine = LearningEngine {
            experiences: vec![], lessons: HashMap::new(), curriculum: Curriculum::new(),
            next_id: 1, next_lesson_id: 1, generalization_threshold: 0.7,
            max_experiences: 500, forgetting_half_life_ms: 86400_000, // 24 hours
        };
        engine.curriculum.add_default();
        engine
    }

    /// Record an experience
    pub fn experience(&mut self, mut exp: Experience) {
        if exp.id == 0 { exp.id = self.next_id; self.next_id += 1; }
        exp.timestamp = now();

        // Extract lesson
        if exp.outcome.abs() > 0.5 {
            let lesson = self.extract_lesson(&exp);
            exp.lesson = lesson.clone();
            exp.generalizable = exp.context.len() > 1;

            if !lesson.is_empty() {
                let existing = self.lessons.entry(lesson.clone()).or_insert_with(|| Lesson {
                    id: self.next_lesson_id, rule: lesson.clone(),
                    confidence: 0.3, success_count: 0, failure_count: 0,
                    contexts: exp.context.clone(), generalization_score: 0.0,
                    created: now(), last_used: 0,
                });
                self.next_lesson_id += 1;
                if exp.outcome > 0.0 { existing.success_count += 1; }
                else { existing.failure_count += 1; }
                existing.confidence = (existing.confidence + 0.1).min(1.0);
                existing.last_used = now();
            }
        }

        if self.experiences.len() >= self.max_experiences { self.experiences.remove(0); }
        self.experiences.push(exp);
    }

    /// Extract lesson from experience
    fn extract_lesson(&self, exp: &Experience) -> String {
        if exp.actions.is_empty() || exp.context.is_empty() { return String::new(); }
        let outcome_word = if exp.outcome > 0.5 { "works well" } else if exp.outcome < -0.5 { "avoid" } else { "try differently" };
        format!("When {} → {} {}", exp.context.join("+"), exp.actions.join("→"), outcome_word)
    }

    /// Query lessons applicable to current context
    pub fn applicable_lessons(&self, context: &[String]) -> Vec<(&String, &Lesson)> {
        self.lessons.iter()
            .map(|(id, lesson)| (id, lesson))
            .filter(|(_, l)| l.apply_confidence(context) > 0.1)
            .collect()
    }

    /// Best lesson for context
    pub fn best_lesson(&self, context: &[String]) -> Option<&Lesson> {
        self.lessons.values()
            .filter(|l| l.apply_confidence(context) > 0.1)
            .max_by(|a, b| a.apply_confidence(context).partial_cmp(&b.apply_confidence(context)).unwrap())
    }

    /// Apply curriculum
    pub fn curriculum_record(&mut self, success: bool) -> Option<String> {
        self.curriculum.record(success)
    }

    /// Forgetting: decay old lessons
    pub fn forget(&mut self, current_time: u64) {
        self.lessons.retain(|_, lesson| {
            let age = current_time.saturating_sub(lesson.last_used);
            let factor = 0.5_f64.powf(age as f64 / self.forgetting_half_life_ms as f64);
            // Keep if recently used or still confident
            factor > 0.1 || lesson.confidence > 0.7
        });
    }

    /// Stats
    pub fn stats(&self) -> LearningStats {
        LearningStats {
            experiences: self.experiences.len(),
            lessons: self.lessons.len(),
            curriculum_stage: self.curriculum.current_stage,
            total_curriculum_stages: self.curriculum.stages.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LearningStats {
    pub experiences: usize,
    pub lessons: usize,
    pub curriculum_stage: usize,
    pub total_curriculum_stages: usize,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for c in s.bytes() { h = h.wrapping_mul(33).wrapping_add(c as u64); }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_creation() {
        let exp = Experience::new(1).with_context(vec!["navigation"]).with_actions(vec!["turn_left"]).with_outcome(0.8);
        assert_eq!(exp.context.len(), 1);
        assert_eq!(exp.actions.len(), 1);
    }

    #[test]
    fn test_temporal_credit() {
        let ca = CreditAssignment::temporal_credit(&["look", "decide", "act"], 1.0);
        assert!(ca.action_credits["act"] > ca.action_credits["look"]); // later gets more credit
    }

    #[test]
    fn test_equal_credit() {
        let ca = CreditAssignment::equal_credit(&["a", "b", "c"], 0.9);
        assert_eq!(ca.action_credits["a"], ca.action_credits["b"]);
    }

    #[test]
    fn test_curriculum_advancement() {
        let mut c = Curriculum::new();
        c.add_stage("easy", 0.2, 0.5, 3);
        c.record(true); c.record(true); c.record(true);
        assert_eq!(c.current_stage, 1); // advanced
    }

    #[test]
    fn test_curriculum_no_advance() {
        let mut c = Curriculum::new();
        c.add_stage("hard", 0.8, 0.9, 10);
        c.record(true); c.record(false);
        assert_eq!(c.current_stage, 0);
    }

    #[test]
    fn test_learning_experience() {
        let mut engine = LearningEngine::new();
        let exp = Experience::new(0).with_context(vec!["navigation", "maze"]).with_actions(vec!["follow_left_wall"]).with_outcome(0.9);
        engine.experience(exp);
        assert_eq!(engine.experiences.len(), 1);
        assert_eq!(engine.lessons.len(), 1); // lesson extracted
    }

    #[test]
    fn test_lesson_applicable() {
        let mut engine = LearningEngine::new();
        engine.experience(Experience::new(0).with_context(vec!["nav", "maze"]).with_actions(vec!["left"]).with_outcome(0.8));
        let lessons = engine.applicable_lessons(&["nav".to_string()]);
        assert!(lessons.len() >= 1);
    }

    #[test]
    fn test_negative_outcome_lesson() {
        let mut engine = LearningEngine::new();
        engine.experience(Experience::new(0).with_context(vec!["combat"]).with_actions(vec!["charge"]).with_outcome(-0.9));
        assert!(engine.lessons.len() == 1);
        let lesson = engine.lessons.values().next().unwrap();
        assert!(lesson.rule.contains("avoid"));
    }

    #[test]
    fn test_forgetting() {
        let mut engine = LearningEngine::new();
        engine.experience(Experience::new(0).with_context(vec!["x"]).with_actions(vec!["y"]).with_outcome(0.8));
        engine.forget(now() + 100_000_000); // far future
        // Lesson should survive since it has high confidence
        // But experiences may be evicted by max_experiences if many added
    }

    #[test]
    fn test_stats() {
        let engine = LearningEngine::new();
        let s = engine.stats();
        assert_eq!(s.lessons, 0);
        assert_eq!(s.experiences, 0);
        assert!(s.total_curriculum_stages > 0);
    }

    #[test]
    fn test_max_experiences() {
        let mut engine = LearningEngine::new();
        engine.max_experiences = 3;
        for i in 0..5 { engine.experience(Experience::new(i).with_context(vec!["x"]).with_actions(vec!["y"]).with_outcome(0.5)); }
        assert_eq!(engine.experiences.len(), 3); // evicted oldest 2
    }

    #[test]
    fn test_lesson_success_rate() {
        let mut lesson = Lesson { id: 1, rule: "test".into(), confidence: 0.5, success_count: 8, failure_count: 2, contexts: vec![], generalization_score: 0.0, created: 0, last_used: 0 };
        assert!((lesson.success_rate() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_context_hash() {
        let a = Experience::new(0).with_context(vec!["nav", "maze"]);
        let b = Experience::new(0).with_context(vec!["nav", "maze"]);
        assert_eq!(a.context_hash, b.context_hash);
    }
}
