//! Usage analytics and insights.
//!
//! Provides analytics calculations and insights generation based on
//! command history data.

use std::time::Duration;

use super::HistoryEntry;

/// Command usage statistics.
#[derive(Debug, Clone)]
pub struct CommandStats {
    /// Command name
    pub name: String,
    /// Total execution count
    pub execution_count: u32,
    /// Success rate (0.0 - 100.0)
    pub success_rate: f64,
    /// Average execution duration
    pub avg_duration: Duration,
    /// Total time spent (execution_count * avg_duration)
    pub total_time: Duration,
}

/// Time period for analytics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePeriod {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// All time
    AllTime,
}

impl TimePeriod {
    /// Get the period as seconds.
    pub fn as_seconds(&self) -> Option<u64> {
        match self {
            Self::Day => Some(86400),
            Self::Week => Some(604800),
            Self::Month => Some(2592000),
            Self::AllTime => None,
        }
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Day => "Today",
            Self::Week => "This Week",
            Self::Month => "This Month",
            Self::AllTime => "All Time",
        }
    }
}

/// Analytics results for display.
#[derive(Debug, Clone, Default)]
pub struct AnalyticsReport {
    /// Time period for this report
    pub period: String,
    /// Total commands executed
    pub total_executions: u32,
    /// Unique commands used
    pub unique_commands: usize,
    /// Overall success rate
    pub overall_success_rate: f64,
    /// Total time spent
    pub total_time: Duration,
    /// Top commands by usage
    pub top_commands: Vec<CommandStats>,
    /// Commands with highest failure rate
    pub failing_commands: Vec<CommandStats>,
    /// Generated insights
    pub insights: Vec<Insight>,
}

/// A generated insight about command usage.
#[derive(Debug, Clone)]
pub struct Insight {
    /// Insight category
    pub category: InsightCategory,
    /// Main message
    pub message: String,
    /// Suggested action (if any)
    pub suggestion: Option<String>,
}

/// Categories of insights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsightCategory {
    /// High usage pattern
    HighUsage,
    /// Performance issue
    Performance,
    /// Failure pattern
    FailureRate,
    /// Time spent
    TimeSpent,
    /// Positive trend
    Positive,
}

impl InsightCategory {
    /// Get icon for this category.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::HighUsage => "📈",
            Self::Performance => "⏱️",
            Self::FailureRate => "⚠️",
            Self::TimeSpent => "⏰",
            Self::Positive => "✨",
        }
    }
}

/// Analytics calculator.
pub struct Analytics;

impl Analytics {
    /// Calculate analytics from history entries.
    pub fn calculate(entries: &[&HistoryEntry], period: TimePeriod) -> AnalyticsReport {
        let period_name = period.display_name().to_string();

        if entries.is_empty() {
            return AnalyticsReport { period: period_name, ..Default::default() };
        }

        // Calculate totals
        let total_executions: u32 = entries.iter().map(|e| e.execution_count).sum();
        let unique_commands = entries.len();
        let total_successes: u32 = entries.iter().map(|e| e.success_count).sum();
        let total_duration_ms: u64 = entries.iter().map(|e| e.total_duration_ms).sum();

        let overall_success_rate = if total_executions > 0 {
            (f64::from(total_successes) / f64::from(total_executions)) * 100.0
        } else {
            0.0
        };

        let total_time = Duration::from_millis(total_duration_ms);

        // Calculate per-command stats
        let mut command_stats: Vec<CommandStats> = entries
            .iter()
            .map(|e| {
                let success_rate = e.success_rate().unwrap_or(0.0);
                let avg_duration = e.average_duration().unwrap_or_default();
                let total_time = Duration::from_millis(e.total_duration_ms);

                CommandStats {
                    name: e.command_name.clone(),
                    execution_count: e.execution_count,
                    success_rate,
                    avg_duration,
                    total_time,
                }
            })
            .collect();

        // Top commands by usage
        command_stats.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        let top_commands: Vec<CommandStats> = command_stats.iter().take(10).cloned().collect();

        // Failing commands (>20% failure rate, sorted by failure rate)
        let mut failing: Vec<CommandStats> = command_stats
            .iter()
            .filter(|s| s.success_rate < 80.0 && s.execution_count >= 3)
            .cloned()
            .collect();
        failing.sort_by(|a, b| {
            a.success_rate.partial_cmp(&b.success_rate).unwrap_or(std::cmp::Ordering::Equal)
        });
        let failing_commands: Vec<CommandStats> = failing.into_iter().take(5).collect();

        // Generate insights
        let insights = Self::generate_insights(&command_stats, total_executions, total_time);

        AnalyticsReport {
            period: period_name,
            total_executions,
            unique_commands,
            overall_success_rate,
            total_time,
            top_commands,
            failing_commands,
            insights,
        }
    }

    /// Generate insights from analytics data.
    fn generate_insights(
        stats: &[CommandStats],
        total_executions: u32,
        total_time: Duration,
    ) -> Vec<Insight> {
        let mut insights = Vec::new();

        // Insight: Most used command
        if let Some(top) = stats.first() {
            if top.execution_count > 10 {
                let percentage =
                    (f64::from(top.execution_count) / f64::from(total_executions)) * 100.0;
                insights.push(Insight {
                    category: InsightCategory::HighUsage,
                    message: format!(
                        "You run '{}' most often ({} times, {:.0}% of all commands)",
                        top.name, top.execution_count, percentage
                    ),
                    suggestion: if percentage > 30.0 {
                        Some("Consider creating an alias for faster access".to_string())
                    } else {
                        None
                    },
                });
            }
        }

        // Insight: High failure rate commands
        let high_failure: Vec<_> =
            stats.iter().filter(|s| s.success_rate < 50.0 && s.execution_count >= 5).collect();

        if let Some(worst) = high_failure.first() {
            insights.push(Insight {
                category: InsightCategory::FailureRate,
                message: format!(
                    "'{}' has a {:.0}% failure rate ({} failures)",
                    worst.name,
                    100.0 - worst.success_rate,
                    (f64::from(worst.execution_count) * (100.0 - worst.success_rate) / 100.0)
                        as u32
                ),
                suggestion: Some("Check the command output for common errors".to_string()),
            });
        }

        // Insight: Time spent
        if total_time.as_secs() > 3600 {
            let hours = total_time.as_secs() as f64 / 3600.0;
            insights.push(Insight {
                category: InsightCategory::TimeSpent,
                message: format!("You've spent {:.1} hours running commands", hours),
                suggestion: None,
            });
        }

        // Insight: Slow commands
        let slow_commands: Vec<_> = stats
            .iter()
            .filter(|s| s.avg_duration.as_secs() > 30 && s.execution_count >= 3)
            .collect();

        if let Some(slowest) = slow_commands.first() {
            insights.push(Insight {
                category: InsightCategory::Performance,
                message: format!(
                    "'{}' takes {:.0}s on average",
                    slowest.name,
                    slowest.avg_duration.as_secs_f64()
                ),
                suggestion: Some("Consider running it in the background (Ctrl+B)".to_string()),
            });
        }

        // Insight: Good success rate
        let good_commands: Vec<_> =
            stats.iter().filter(|s| s.success_rate >= 95.0 && s.execution_count >= 10).collect();

        if good_commands.len() >= 3 {
            insights.push(Insight {
                category: InsightCategory::Positive,
                message: format!("{} commands have a 95%+ success rate", good_commands.len()),
                suggestion: None,
            });
        }

        insights
    }

    /// Generate a bar chart representation for TUI.
    ///
    /// Returns lines of text representing a horizontal bar chart.
    pub fn bar_chart(stats: &[CommandStats], max_width: usize) -> Vec<String> {
        if stats.is_empty() {
            return vec!["No data".to_string()];
        }

        let max_count = stats.iter().map(|s| s.execution_count).max().unwrap_or(1);
        let max_name_len = stats.iter().map(|s| s.name.len()).max().unwrap_or(10).min(20);

        stats
            .iter()
            .take(10)
            .map(|s| {
                let name = if s.name.len() > max_name_len {
                    format!("{}...", &s.name[..max_name_len - 3])
                } else {
                    format!("{:width$}", s.name, width = max_name_len)
                };

                let bar_max_width = max_width.saturating_sub(max_name_len + 10);
                let bar_len = (f64::from(s.execution_count) / f64::from(max_count)
                    * bar_max_width as f64) as usize;
                let bar = "█".repeat(bar_len.max(1));

                format!("{} {} ({})", name, bar, s.execution_count)
            })
            .collect()
    }

    /// Format duration for display.
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(name: &str, count: u32, successes: u32, duration_ms: u64) -> HistoryEntry {
        HistoryEntry {
            command_id: name.to_string(),
            command_name: name.to_string(),
            execution_count: count,
            last_executed: 0,
            first_executed: 0,
            total_duration_ms: duration_ms,
            success_count: successes,
            failure_count: count - successes,
        }
    }

    #[test]
    fn test_analytics_empty() {
        let report = Analytics::calculate(&[], TimePeriod::AllTime);
        assert_eq!(report.total_executions, 0);
        assert_eq!(report.unique_commands, 0);
    }

    #[test]
    fn test_analytics_basic() {
        let entry1 = create_test_entry("npm test", 10, 8, 5000);
        let entry2 = create_test_entry("npm build", 5, 5, 10000);
        let entries: Vec<&HistoryEntry> = vec![&entry1, &entry2];

        let report = Analytics::calculate(&entries, TimePeriod::AllTime);

        assert_eq!(report.total_executions, 15);
        assert_eq!(report.unique_commands, 2);
        assert!((report.overall_success_rate - 86.67).abs() < 1.0);
    }

    #[test]
    fn test_top_commands() {
        let entry1 = create_test_entry("npm test", 20, 18, 5000);
        let entry2 = create_test_entry("npm build", 5, 5, 10000);
        let entry3 = create_test_entry("npm lint", 15, 15, 3000);
        let entries: Vec<&HistoryEntry> = vec![&entry1, &entry2, &entry3];

        let report = Analytics::calculate(&entries, TimePeriod::AllTime);

        assert_eq!(report.top_commands.len(), 3);
        assert_eq!(report.top_commands[0].name, "npm test");
        assert_eq!(report.top_commands[1].name, "npm lint");
        assert_eq!(report.top_commands[2].name, "npm build");
    }

    #[test]
    fn test_failing_commands() {
        let entry1 = create_test_entry("npm test", 10, 3, 5000); // 30% success
        let entry2 = create_test_entry("npm build", 10, 9, 10000); // 90% success
        let entries: Vec<&HistoryEntry> = vec![&entry1, &entry2];

        let report = Analytics::calculate(&entries, TimePeriod::AllTime);

        assert_eq!(report.failing_commands.len(), 1);
        assert_eq!(report.failing_commands[0].name, "npm test");
    }

    #[test]
    fn test_bar_chart() {
        let stats = vec![
            CommandStats {
                name: "npm test".to_string(),
                execution_count: 100,
                success_rate: 90.0,
                avg_duration: Duration::from_secs(5),
                total_time: Duration::from_secs(500),
            },
            CommandStats {
                name: "npm build".to_string(),
                execution_count: 50,
                success_rate: 100.0,
                avg_duration: Duration::from_secs(30),
                total_time: Duration::from_secs(1500),
            },
        ];

        let chart = Analytics::bar_chart(&stats, 50);
        assert_eq!(chart.len(), 2);
        assert!(chart[0].contains("npm test"));
        assert!(chart[0].contains("(100)"));
        assert!(chart[1].contains("npm build"));
        assert!(chart[1].contains("(50)"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(Analytics::format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(Analytics::format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(Analytics::format_duration(Duration::from_secs(3725)), "1h 2m");
    }

    #[test]
    fn test_time_period() {
        assert_eq!(TimePeriod::Day.as_seconds(), Some(86400));
        assert_eq!(TimePeriod::Week.as_seconds(), Some(604800));
        assert_eq!(TimePeriod::AllTime.as_seconds(), None);
        assert_eq!(TimePeriod::Day.display_name(), "Today");
    }

    #[test]
    fn test_insight_generation() {
        let entry = create_test_entry("npm test", 50, 20, 100000); // 40% success
        let entries: Vec<&HistoryEntry> = vec![&entry];

        let report = Analytics::calculate(&entries, TimePeriod::AllTime);

        // Should have insights about high usage and failure rate
        assert!(!report.insights.is_empty());
    }
}
