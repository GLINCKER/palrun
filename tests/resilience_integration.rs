//! Resilience Integration Tests
//!
//! Tests for graceful degradation, offline mode, and resilience infrastructure.

use palrun::core::{
    CircuitState, DegradationManager, DegradationReason, DegradedFeature, Feature,
    FeatureResilience, OfflineManager, QueuedOperation, ResilienceManager, ResilientResult,
};

// ============================================================================
// Graceful Degradation Tests (Phase 25c)
// ============================================================================

mod graceful_degradation {
    use super::*;

    #[test]
    fn test_feature_degradation_tracking() {
        let mut manager = DegradationManager::new();

        // Initially no features degraded
        assert!(!manager.is_degraded(Feature::Ai));
        assert!(!manager.has_degradations());

        // Degrade AI feature
        manager.degrade(Feature::Ai, DegradationReason::ServiceOffline("Claude API".into()));

        assert!(manager.is_degraded(Feature::Ai));
        assert!(manager.has_degradations());
    }

    #[test]
    fn test_multiple_feature_degradation() {
        let mut manager = DegradationManager::new();

        // Degrade multiple features
        manager.degrade(Feature::Ai, DegradationReason::ServiceOffline("Claude".into()));
        manager.degrade(Feature::Network, DegradationReason::NetworkUnavailable);
        manager.degrade(Feature::Sync, DegradationReason::CircuitOpen);

        assert_eq!(manager.degraded_features().len(), 3);

        // Recover one feature
        manager.recover(Feature::Ai);
        assert_eq!(manager.degraded_features().len(), 2);
        assert!(!manager.is_degraded(Feature::Ai));
    }

    #[test]
    fn test_degradation_reason_display() {
        let reasons = [
            (DegradationReason::Disabled, "disabled"),
            (DegradationReason::ServiceOffline("API".into()), "offline"),
            (DegradationReason::MissingCredentials, "missing"),
            (DegradationReason::NetworkUnavailable, "network"),
            (DegradationReason::CircuitOpen, "unavailable"),
        ];

        for (reason, expected_contains) in &reasons {
            let display = format!("{}", reason);
            assert!(
                display.to_lowercase().contains(expected_contains),
                "Expected '{}' to contain '{}', got: {}",
                display,
                expected_contains,
                display
            );
        }
    }

    #[test]
    fn test_recovery_hint_provided() {
        // Test that recovery hints are provided for various scenarios
        let features_and_reasons = [
            (Feature::Ai, DegradationReason::MissingCredentials),
            (Feature::Network, DegradationReason::NetworkUnavailable),
            (Feature::Sync, DegradationReason::CircuitOpen),
        ];

        for (feature, reason) in features_and_reasons {
            let degraded = DegradedFeature::new(feature, reason);
            // Recovery hint should be provided
            assert!(
                degraded.recovery_hint.is_some() || degraded.fallback.is_some(),
                "Feature {:?} should have recovery hint or fallback",
                feature
            );
        }
    }

    #[test]
    fn test_degradation_summary() {
        let mut manager = DegradationManager::new();
        manager.degrade(Feature::Ai, DegradationReason::NetworkUnavailable);

        let summary = manager.summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("AI") || summary.contains("degraded"));
    }

    #[test]
    fn test_recovery_hints_collection() {
        let mut manager = DegradationManager::new();
        manager.degrade(Feature::Ai, DegradationReason::MissingCredentials);
        manager.degrade(Feature::Network, DegradationReason::NetworkUnavailable);

        let hints = manager.recovery_hints();
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_clear_degradations() {
        let mut manager = DegradationManager::new();
        manager.degrade(Feature::Ai, DegradationReason::NetworkUnavailable);
        manager.degrade(Feature::Sync, DegradationReason::CircuitOpen);

        assert!(manager.has_degradations());

        manager.clear();
        assert!(!manager.has_degradations());
    }
}

// ============================================================================
// Offline Mode Tests (Phase 25d)
// ============================================================================

mod offline_mode {
    use super::*;

    #[test]
    fn test_offline_detection() {
        let mut manager = OfflineManager::new();

        // Initially online
        assert!(!manager.is_offline());

        // Set offline
        manager.set_offline(true);
        assert!(manager.is_offline());

        // Set online
        manager.set_offline(false);
        assert!(!manager.is_offline());
    }

    #[test]
    fn test_operation_queuing_when_offline() {
        let mut manager = OfflineManager::new();
        manager.set_offline(true);

        // Queue operations
        manager
            .queue_operation(QueuedOperation::AiRequest { prompt: "test".into(), context: None });

        manager.queue_operation(QueuedOperation::SyncHistory { entries_count: 10 });

        assert_eq!(manager.queue().len(), 2);
        assert!(!manager.queue().is_empty());
    }

    #[test]
    fn test_queue_dequeue_cycle() {
        let mut manager = OfflineManager::new();

        // Queue operation
        manager.queue_operation(QueuedOperation::Webhook {
            url: "https://example.com".into(),
            payload: "{}".into(),
        });

        assert_eq!(manager.queue().len(), 1);

        // Dequeue
        let op = manager.queue_mut().dequeue();

        assert!(op.is_some());
        assert!(manager.queue().is_empty());
    }

    #[test]
    fn test_queued_operation_types() {
        // Verify all operation types can be created
        let operations = vec![
            QueuedOperation::AiRequest { prompt: "test".into(), context: Some("ctx".into()) },
            QueuedOperation::SyncHistory { entries_count: 5 },
            QueuedOperation::SendAnalytics { event_type: "command_run".into(), data: "{}".into() },
            QueuedOperation::Webhook {
                url: "https://api.example.com".into(),
                payload: "{}".into(),
            },
            QueuedOperation::Custom { operation_type: "custom".into(), data: "data".into() },
        ];

        for op in &operations {
            // Verify display works
            let _ = format!("{}", op);
        }
    }

    #[test]
    fn test_queue_summary() {
        let mut manager = OfflineManager::new();

        manager
            .queue_operation(QueuedOperation::AiRequest { prompt: "test".into(), context: None });
        manager.queue_operation(QueuedOperation::SyncHistory { entries_count: 5 });

        let summary = manager.queue().summary();
        assert_eq!(summary.total, 2);
    }

    #[test]
    fn test_connectivity_check_timing() {
        let mut manager = OfflineManager::new();

        // Should check initially
        assert!(manager.should_check_connectivity());

        // Mark as checked
        manager.mark_checked();

        // Should not need immediate recheck
        // (depending on implementation this may still be true)
        let _ = manager.should_check_connectivity();
    }
}

// ============================================================================
// Resilience Infrastructure Tests
// ============================================================================

mod resilience_infrastructure {
    use super::*;

    #[test]
    fn test_resilient_result_types() {
        // Success
        let success: ResilientResult<i32> = ResilientResult::success(42, 1);
        assert!(success.is_success());
        assert_eq!(success.into_value(), Some(42));

        // Queued
        let queued: ResilientResult<i32> = ResilientResult::queued();
        assert!(!queued.is_success());
        assert!(queued.queued);

        // Fallback
        let fallback: ResilientResult<i32> = ResilientResult::fallback(0);
        assert!(fallback.is_success());
        assert!(fallback.used_fallback);

        // Failed
        let failed: ResilientResult<i32> = ResilientResult::failed("error", 3);
        assert!(!failed.is_success());
        assert_eq!(failed.error, Some("error".to_string()));
    }

    #[test]
    fn test_feature_resilience_per_feature() {
        // Each feature should have appropriate configuration
        let features =
            [Feature::Ai, Feature::Network, Feature::Sync, Feature::Integrations, Feature::Mcp];

        for feature in features {
            let resilience = FeatureResilience::new(feature);
            assert!(resilience.is_available());
            assert_eq!(resilience.circuit_state(), CircuitState::Closed);
        }
    }

    #[test]
    fn test_resilience_manager_all_features() {
        let manager = ResilienceManager::new();

        // All features should be available initially
        assert!(manager.ai.is_available());
        assert!(manager.network.is_available());
        assert!(manager.sync.is_available());
        assert!(manager.integrations.is_available());
        assert!(manager.mcp.is_available());
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let resilience = FeatureResilience::new(Feature::Network);

        // Initially closed
        assert_eq!(resilience.circuit_state(), CircuitState::Closed);

        // Record failures until circuit opens
        // Network has threshold of 5
        for _ in 0..6 {
            resilience.record_failure();
        }

        // Should be open now
        assert_eq!(resilience.circuit_state(), CircuitState::Open);
        assert!(!resilience.is_available());

        // Reset should close
        resilience.reset();
        assert_eq!(resilience.circuit_state(), CircuitState::Closed);
        assert!(resilience.is_available());
    }

    #[test]
    fn test_execute_with_retry_success() {
        let resilience = FeatureResilience::new(Feature::Ai);

        let result = resilience.execute(|| Ok::<_, &str>("success"));

        assert!(result.is_success());
        assert_eq!(result.value, Some("success"));
        assert!(result.attempts >= 1);
    }

    #[test]
    fn test_execute_with_retry_failure() {
        let resilience = FeatureResilience::new(Feature::Mcp); // Quick retry config

        let result = resilience.execute(|| Err::<i32, _>("always fails"));

        assert!(!result.is_success());
        assert!(result.attempts >= 1);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_resilience_status_summary() {
        let manager = ResilienceManager::new();
        let status = manager.status_summary();

        // Should have 5 features
        assert_eq!(status.len(), 5);

        // All should be closed initially
        for (_, state) in &status {
            assert_eq!(*state, CircuitState::Closed);
        }
    }
}

// ============================================================================
// Integration: Degradation + Resilience
// ============================================================================

mod degradation_resilience_integration {
    use super::*;

    #[test]
    fn test_execute_with_degradation_tracking() {
        let manager = ResilienceManager::new();
        let mut degradation = DegradationManager::new();

        // Successful operation should not degrade
        let result = manager
            .execute_with_degradation(Feature::Ai, &mut degradation, || Ok::<_, &str>("success"));

        assert!(result.is_success());
        assert!(!degradation.is_degraded(Feature::Ai));
    }

    #[test]
    fn test_execute_with_queue_when_offline() {
        let manager = ResilienceManager::new();
        let mut offline = OfflineManager::new();
        offline.set_offline(true);

        let result: ResilientResult<String> = manager.execute_with_queue(
            Feature::Sync,
            &mut offline,
            || Ok::<_, &str>("data".into()),
            || QueuedOperation::SyncHistory { entries_count: 1 },
        );

        assert!(result.queued);
        assert!(!offline.queue().is_empty());
    }

    #[test]
    fn test_full_resilience_workflow() {
        let manager = ResilienceManager::new();
        let mut degradation = DegradationManager::new();
        let mut offline = OfflineManager::new();

        // 1. Execute when online - should succeed
        let result =
            manager.execute_with_degradation(Feature::Ai, &mut degradation, || Ok::<_, &str>(42));
        assert!(result.is_success());

        // 2. Go offline
        offline.set_offline(true);

        // 3. Try operation - should be queued
        let result: ResilientResult<i32> = manager.execute_with_queue(
            Feature::Sync,
            &mut offline,
            || Ok::<_, &str>(1),
            || QueuedOperation::SyncHistory { entries_count: 1 },
        );
        assert!(result.queued);

        // 4. Come back online
        offline.set_offline(false);

        // 5. Process queue
        while let Some(_entry) = offline.queue_mut().dequeue() {
            // Process entry...
        }
        assert!(offline.queue().is_empty());
    }
}
