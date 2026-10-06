# Parallel-fast amendment implementation evidence

Approved by owner on 2026-10-05. Three focused new contracts and the recipe mode assertion observed RED→GREEN. Both modes exercise interruption during an executing cleanup and an unwound case, and retain unexpected-success failure. Host and Ubuntu compare all 320 native IDs and exact 37/40 fast skips; full CI remains 320 cases with five permitted skips. Rust affected selector selected zero packages.

## amend-fast-red.log

```text
FFFFF
======================================================================
FAIL: test_fast_preserves_native_inventory_and_module_fixtures (tools.test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures) (budget='10')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 130, in test_fast_preserves_native_inventory_and_module_fixtures
    self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: 2 != 0 : usage: python3 -m tools.tooling_tests [-h] (--full | --check-paths) [--ci]
python3 -m tools.tooling_tests: error: one of the arguments --full --check-paths is required


======================================================================
FAIL: test_fast_preserves_native_inventory_and_module_fixtures (tools.test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures) (budget='1')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 130, in test_fast_preserves_native_inventory_and_module_fixtures
    self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: 2 != 0 : usage: python3 -m tools.tooling_tests [-h] (--full | --check-paths) [--ci]
python3 -m tools.tooling_tests: error: one of the arguments --full --check-paths is required


======================================================================
FAIL: test_fast_rejects_ci_and_public_worker_mode_before_discovery (tools.test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery) (args=['--fast', '--ci'])
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 157, in test_fast_rejects_ci_and_public_worker_mode_before_discovery
    self.assertIn(message, run.stderr)
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^
AssertionError: '--ci requires --full' not found in 'usage: python3 -m tools.tooling_tests [-h] (--full | --check-paths) [--ci]\npython3 -m tools.tooling_tests: error: one of the arguments --full --check-paths is required\n'

======================================================================
FAIL: test_fast_rejects_ci_and_public_worker_mode_before_discovery (tools.test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery) (args=['--full', '--worker-mode', '1'])
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 157, in test_fast_rejects_ci_and_public_worker_mode_before_discovery
    self.assertIn(message, run.stderr)
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^
AssertionError: '--worker-mode requires --worker' not found in 'usage: python3 -m tools.tooling_tests [-h] (--full | --check-paths) [--ci]\npython3 -m tools.tooling_tests: error: unrecognized arguments: --worker-mode\n'

======================================================================
FAIL: test_fast_malformed_budget_fails_before_discovery (tools.test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_tooling_tests.py", line 165, in test_fast_malformed_budget_fails_before_discovery
    self.assertIn('NEXTEST_TEST_THREADS', run.stderr)
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: 'NEXTEST_TEST_THREADS' not found in 'usage: python3 -m tools.tooling_tests [-h] (--full | --check-paths) [--ci]\npython3 -m tools.tooling_tests: error: one of the arguments --full --check-paths is required\n'

----------------------------------------------------------------------
Ran 3 tests in 0.476s

FAILED (failures=5)
error: recipe `test-one` failed on line 63 with exit code 1
```

## amend-recipe-red.log

```text
FFF
======================================================================
FAIL: test_recipe_composition_modes_counts_and_shared_hook_target (tools.test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target) (recipe='check')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_gates.py", line 257, in test_recipe_composition_modes_counts_and_shared_hook_target
    self.assertIn('--fast', event['args'])
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: '--fast' not found in ['discover', '-s', 'tools']

======================================================================
FAIL: test_recipe_composition_modes_counts_and_shared_hook_target (tools.test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target) (recipe='hook-pre-commit')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_gates.py", line 257, in test_recipe_composition_modes_counts_and_shared_hook_target
    self.assertIn('--fast', event['args'])
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: '--fast' not found in ['discover', '-s', 'tools']

======================================================================
FAIL: test_recipe_composition_modes_counts_and_shared_hook_target (tools.test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target) (recipe='hook-pre-push-fast')
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tools/test_gates.py", line 257, in test_recipe_composition_modes_counts_and_shared_hook_target
    self.assertIn('--fast', event['args'])
    ~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: '--fast' not found in ['discover', '-s', 'tools']

----------------------------------------------------------------------
Ran 1 test in 1.236s

FAILED (failures=3)
error: recipe `test-one` failed on line 63 with exit code 1
```

## amend-focus-green.log

```text
.................................
----------------------------------------------------------------------
Ran 33 tests in 8.087s

OK
```

## amend-cancel-green.log

```text
.....................
----------------------------------------------------------------------
Ran 21 tests in 4.369s

OK
```

## amend-full-green.log

```text
Full tooling: 320 cases, 10 children
case test_affected.Select.test_docs_and_tasks_select_nothing: 0.000s
case test_affected.Select.test_excluded_member_is_never_selected: 0.000s
case test_affected.Select.test_member_change_selects_its_dependents: 0.000s
case test_affected.Select.test_nothing_changed_selects_nothing: 0.000s
case test_affected.Select.test_root_source_selects_the_root_package: 0.000s
case test_affected.Select.test_workspace_files_select_everything: 0.000s
case test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi: 2.674s
case test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry: 0.000s
case test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty: 0.004s
case test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run: 0.000s
case test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run: 0.000s
case test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint: 0.002s
case test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run: 0.016s
case test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process: 0.007s
case test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash: 0.034s
case test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing: 0.003s
case test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source: 0.019s
case test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing: 0.001s
case test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing: 0.011s
case test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash: 0.037s
case test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames: 0.021s
case test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text: 0.034s
case test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses: 0.001s
case test_capture_meta.JudgementTests.test_quiet_and_settle: 0.000s
case test_capture_meta.JudgementTests.test_threshold_validation: 0.000s
case test_capture_meta.LockTests.test_acquire_release_and_ownership_check: 0.001s
case test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder: 0.301s
case test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed: 0.006s
case test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed: 0.001s
case test_capture_meta.LockTests.test_public_read_waits_for_guard: 0.301s
case test_capture_meta.LockTests.test_rejects_invalid_owner_fields: 0.006s
case test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one: 0.117s
case test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients: 0.005s
case test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run: 0.001s
case test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load: 0.007s
case test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients: 0.009s
case test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu: 0.003s
case test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling: 0.002s
case test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock: 0.003s
case test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run: 0.001s
case test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock: 0.003s
case test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight: 0.003s
case test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load: 0.003s
case test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order: 0.000s
case test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list: 0.000s
case test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run: 0.000s
case test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema: 0.000s
case test_capture_meta.RecordTests.test_load_record_rejects_non_object_json: 0.000s
case test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation: 0.365s
case test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite: 0.001s
case test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks: 0.006s
case test_capture_meta.SamplingTests.test_sample_stream_and_summary: 0.000s
case test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients: 0.000s
case test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample: 0.007s
case test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse: 0.007s
case test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises: 0.007s
case test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock: 0.004s
case test_capture_meta.SettleTests.test_release_command_is_ownership_checked: 0.007s
case test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses: 0.004s
case test_capture_meta.SettleTests.test_settled_entry_carries_inputs: 0.004s
case test_capture_meta.ShowTests.test_main_show_exit_codes: 0.002s
case test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts: 0.000s
case test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full: 0.230s
case test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full: 0.218s
case test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate: 0.023s
case test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr: 0.110s
case test_gates.Gates.test_focused_recipe_records_failure_through_host_budget: 0.100s
case test_gates.Gates.test_focused_recipe_requires_arguments: 0.009s
case test_gates.Gates.test_full_recipe_failure_stops_before_following_stages: 0.096s
case test_gates.Gates.test_lfs_failure_stops_before_gate: 0.054s
case test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast: 1.575s
case test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input: 0.343s
case test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target: 1.204s
case test_gates.Gates.test_staged_paths_select_docs_only_conservatively: 0.732s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal: 0.004s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest: 0.014s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles: 1.088s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls: 0.080s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments: 0.005s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name: 0.006s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub: 0.007s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build: 0.000s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights: 0.013s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch: 0.015s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight: 0.005s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build: 0.003s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait: 0.008s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs: 0.275s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: 0.051s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup: 0.022s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: 0.555s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order: 0.002s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound: 0.080s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture: 0.003s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach: 0.019s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam: 0.001s
case test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces: 0.041s
case test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail: 0.051s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: 1.259s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals: 0.006s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge: 0.267s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta: 0.001s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry: 0.217s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light: 0.020s
case test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time: 0.000s
case test_optic_settling.EdgeTests.test_window_counts_half_open_intervals: 0.000s
case test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only: 0.003s
case test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup: 0.007s
case test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed: 0.172s
case test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane: 0.306s
case test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime: 0.467s
case test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case: 0.007s
case test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval: 0.004s
case test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request: 0.008s
case test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked: 0.014s
case test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws: 0.009s
case test_optic_settling.RunTests.test_collect_frame_must_redraw: 0.005s
case test_optic_settling.RunTests.test_complete_lane_passes: 0.011s
case test_optic_settling.RunTests.test_complete_pilot_passes: 0.002s
case test_optic_settling.RunTests.test_consumer_frames_and_samples: 0.005s
case test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale: 0.014s
case test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot: 0.004s
case test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus: 0.005s
case test_optic_settling.RunTests.test_journal_needs_consistent_alignment: 0.002s
case test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases: 0.005s
case test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log: 0.003s
case test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end: 0.006s
case test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls: 0.007s
case test_optic_settling.RunTests.test_rejects_heartbeat_gap: 0.002s
case test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries: 0.010s
case test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases: 0.011s
case test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture: 0.005s
case test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples: 0.006s
case test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace: 0.002s
case test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause: 0.004s
case test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval: 0.003s
case test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window: 0.005s
case test_optic_settling.RunTests.test_trace_without_messages: 0.004s
case test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return: 0.005s
case test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason: 0.003s
case test_optic_settling.RunTests.test_zone_names_with_unquoted_commas: 0.003s
case test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit: 0.128s
case test_package_pin.PinTest.test_check_rejects_a_hand_edited_count: 0.118s
case test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline: 0.061s
case test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values: 0.068s
case test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary: 0.477s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero: 0.358s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error: 0.383s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error: 0.431s
case test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session: 0.470s
case test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero: 0.356s
case test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged: 0.000s
case test_screencast_consumer.PackedRgbTests.test_short_buffer_raises: 0.000s
case test_screencast_consumer.PackedRgbTests.test_strips_row_padding: 0.000s
case test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl: 0.000s
case test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending: 0.000s
case test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request: 0.007s
case test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending: 0.000s
case test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last: 0.001s
case test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request: 0.001s
case test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption: 0.754s
case test_screencast_consumer.WaitForTests.test_times_out_without_a_node: 0.201s
case test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared: 0.201s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped: 0.266s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped: 0.178s
case test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named: 0.198s
case test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named: 0.156s
case test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped: 0.260s
case test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes: 0.171s
case test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped: 0.240s
case test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory: 0.291s
case test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared: 0.163s
case test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config: 0.440s
case test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run: 0.124s
case test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail: 0.412s
case test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails: 0.321s
case test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass: 0.429s
case test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution: 0.082s
case test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail: 0.657s
case test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting: 0.160s
case test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery: 0.081s
case test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures: 0.418s
case test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery: 0.166s
case test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids: 0.270s
case test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping: 1.333s
case test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case: 0.363s
case test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn: 0.081s
case test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted: 0.161s
case test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure: 0.325s
case test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values: 0.022s
case test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids: 0.000s
case test_tooling_tests.ModeTests.test_modes_fail_early: 0.015s
case test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping: 0.160s
case test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail: 0.024s
case test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers: 0.003s
case test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed: 0.020s
case test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd: 0.007s
case test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed: 0.011s
case test_upstream_report.Baseline.test_fork_tree_mismatch_fails: 0.123s
case test_upstream_report.Baseline.test_merge_base_is_never_consulted: 0.119s
case test_upstream_report.Baseline.test_missing_carried_key_fails: 0.081s
case test_upstream_report.Baseline.test_recorded_tree_mismatch_fails: 0.102s
case test_upstream_report.Baseline.test_resolution_needs_no_branches: 0.111s
case test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates: 0.119s
case test_upstream_report.Baseline.test_tag_naming_another_tree_fails: 0.101s
case test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates: 0.135s
case test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail: 0.107s
case test_upstream_report.Classify.test_added_scaffolding_is_class_c: 0.000s
case test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_modified_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_other_additions_are_class_a: 0.000s
case test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one: 0.294s
case test_upstream_report.Cli.test_drift_conflict_exits_one: 0.244s
case test_upstream_report.Cli.test_fresh_report_exits_zero: 0.291s
case test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback: 0.175s
case test_upstream_report.Cli.test_missing_markers_exit_two: 0.182s
case test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits: 0.495s
case test_upstream_report.Cli.test_stale_report_exits_one: 0.195s
case test_upstream_report.Cli.test_truncated_hash_exits_two: 0.177s
case test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one: 0.020s
case test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback: 0.320s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away: 0.078s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one: 0.082s
case test_upstream_report.Conflicts.test_acknowledged_path_passes: 0.018s
case test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result: 0.072s
case test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts: 0.052s
case test_upstream_report.Conflicts.test_clean_status_never_produces_findings: 0.016s
case test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails: 0.018s
case test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths: 0.076s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict: 0.085s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict: 0.079s
case test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries: 0.078s
case test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses: 0.018s
case test_upstream_report.Conflicts.test_malformed_informational_section_raises: 0.016s
case test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding: 0.017s
case test_upstream_report.Conflicts.test_unattributed_conflict_fails: 0.018s
case test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path: 0.017s
case test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag: 0.152s
case test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename: 0.125s
case test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit: 0.101s
case test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name: 0.097s
case test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit: 0.148s
case test_upstream_report.Freshness.test_generation_is_a_fixpoint: 0.132s
case test_upstream_report.Freshness.test_regenerated_and_staged_report_passes: 0.141s
case test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose: 0.133s
case test_upstream_report.Freshness.test_stale_report_is_a_finding: 0.134s
case test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored: 0.156s
case test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict: 0.139s
case test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error: 0.096s
case test_upstream_report.Inventory.test_inventory_excludes_task_records: 0.114s
case test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files: 0.112s
case test_upstream_report.Inventory.test_inventory_is_sorted_by_path: 0.101s
case test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree: 0.106s
case test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts: 0.108s
case test_upstream_report.Inventory.test_local_block_counts_classes: 0.137s
case test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path: 0.096s
case test_upstream_report.Splice.test_markers_out_of_order_are_an_error: 0.000s
case test_upstream_report.Splice.test_missing_markers_are_an_error: 0.000s
case test_upstream_report.Splice.test_splice_is_idempotent: 0.000s
case test_upstream_report.Splice.test_splice_replaces_only_between_markers: 0.000s
case test_upstream_report.Stage.test_fresh_report_is_left_alone: 0.140s
case test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale: 0.174s
case test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged: 0.150s
case test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes: 0.132s
case test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report: 0.144s
case test_vdrag.WireTest.test_fixed_is_24_8: 0.000s
case test_vdrag.WireTest.test_header_packs_object_size_and_opcode: 0.000s
case test_vdrag.WireTest.test_parse_global: 0.000s
case test_vdrag.WireTest.test_split_messages_handles_partial_tail: 0.000s
case test_vdrag.WireTest.test_string_is_padded_and_nul_terminated: 0.000s
case test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: 19.228s
case test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: 1.176s
case test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: 16.471s
case test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: 12.083s
case test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: 16.526s
case test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: 1.698s
case test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: 6.584s
case test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: 8.805s
case test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: 3.205s
case test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: 1.585s
case test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: 0.114s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: 12.240s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: 9.993s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: 10.125s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: 13.376s
case test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: 0.224s
case test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: 0.212s
case test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: 4.727s
case test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: 3.356s
case test_optic_settling.DriverCleanupTests.test_term_during_export: 15.103s
case test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: 12.188s
case test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: 5.532s
case test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: 5.479s
case test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: 0.303s
case test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: 8.026s
case test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: 6.346s
case test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: 6.354s
case test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: 0.012s
case test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: 0.032s
case test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: 0.020s
case test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: 0.025s
case test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: 0.017s
case test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: 0.068s
case test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: 2.126s
case test_vt_lib.VtLibTests.test_term_while_away_restores_home: 0.068s
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL

Ran 320 tests in 33.578s
OK (skipped=2)
```

## amend-fast-host.log

```text
Fast tooling: 320 cases, 2 children
case test_upstream_report.Baseline.test_fork_tree_mismatch_fails: 0.106s
case test_upstream_report.Baseline.test_merge_base_is_never_consulted: 0.116s
case test_upstream_report.Baseline.test_missing_carried_key_fails: 0.064s
case test_upstream_report.Baseline.test_recorded_tree_mismatch_fails: 0.083s
case test_upstream_report.Baseline.test_resolution_needs_no_branches: 0.106s
case test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates: 0.096s
case test_upstream_report.Baseline.test_tag_naming_another_tree_fails: 0.082s
case test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates: 0.131s
case test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail: 0.097s
case test_upstream_report.Classify.test_added_scaffolding_is_class_c: 0.000s
case test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_modified_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_other_additions_are_class_a: 0.000s
case test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one: 0.298s
case test_upstream_report.Cli.test_drift_conflict_exits_one: 0.234s
case test_upstream_report.Cli.test_fresh_report_exits_zero: 0.264s
case test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback: 0.171s
case test_upstream_report.Cli.test_missing_markers_exit_two: 0.211s
case test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits: 0.463s
case test_upstream_report.Cli.test_stale_report_exits_one: 0.183s
case test_upstream_report.Cli.test_truncated_hash_exits_two: 0.173s
case test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one: 0.020s
case test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback: 0.324s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away: 0.073s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one: 0.057s
case test_upstream_report.Conflicts.test_acknowledged_path_passes: 0.018s
case test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result: 0.065s
case test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts: 0.048s
case test_upstream_report.Conflicts.test_clean_status_never_produces_findings: 0.017s
case test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails: 0.012s
case test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths: 0.068s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict: 0.074s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict: 0.068s
case test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries: 0.072s
case test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses: 0.021s
case test_upstream_report.Conflicts.test_malformed_informational_section_raises: 0.020s
case test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding: 0.019s
case test_upstream_report.Conflicts.test_unattributed_conflict_fails: 0.018s
case test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path: 0.015s
case test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag: 0.147s
case test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename: 0.128s
case test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit: 0.121s
case test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name: 0.108s
case test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit: 0.134s
case test_upstream_report.Freshness.test_generation_is_a_fixpoint: 0.121s
case test_upstream_report.Freshness.test_regenerated_and_staged_report_passes: 0.140s
case test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose: 0.120s
case test_upstream_report.Freshness.test_stale_report_is_a_finding: 0.134s
case test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored: 0.173s
case test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict: 0.109s
case test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error: 0.093s
case test_upstream_report.Inventory.test_inventory_excludes_task_records: 0.108s
case test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files: 0.104s
case test_upstream_report.Inventory.test_inventory_is_sorted_by_path: 0.101s
case test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree: 0.107s
case test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts: 0.100s
case test_upstream_report.Inventory.test_local_block_counts_classes: 0.121s
case test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path: 0.100s
case test_upstream_report.Splice.test_markers_out_of_order_are_an_error: 0.000s
case test_upstream_report.Splice.test_missing_markers_are_an_error: 0.000s
case test_upstream_report.Splice.test_splice_is_idempotent: 0.000s
case test_upstream_report.Splice.test_splice_replaces_only_between_markers: 0.000s
case test_upstream_report.Stage.test_fresh_report_is_left_alone: 0.136s
case test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale: 0.170s
case test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged: 0.140s
case test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes: 0.130s
case test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report: 0.141s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal: 0.004s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest: 0.013s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles: 1.204s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls: 0.095s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments: 0.006s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name: 0.009s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub: 0.007s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build: 0.001s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights: 0.017s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch: 0.017s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build: 0.004s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait: 0.008s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median: 0.009s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace: 0.008s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs: 0.356s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: 0.056s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup: 0.028s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope: 0.010s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: 0.569s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order: 0.004s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound: 0.096s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture: 0.004s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach: 0.023s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam: 0.000s
case test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces: 0.045s
case test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail: 0.061s
case test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution: 0.086s
case test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail: 0.563s
case test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting: 0.161s
case test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery: 0.081s
case test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures: 0.411s
case test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery: 0.158s
case test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids: 0.257s
case test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping: 1.350s
case test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case: 0.389s
case test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn: 0.074s
case test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted: 0.183s
case test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure: 0.320s
case test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values: 0.013s
case test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids: 0.000s
case test_tooling_tests.ModeTests.test_modes_fail_early: 0.009s
case test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping: 0.167s
case test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail: 0.025s
case test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers: 0.003s
case test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed: 0.020s
case test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd: 0.006s
case test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed: 0.011s
case test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared: 0.171s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped: 0.279s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped: 0.199s
case test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named: 0.169s
case test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named: 0.193s
case test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped: 0.268s
case test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes: 0.182s
case test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped: 0.251s
case test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory: 0.304s
case test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared: 0.177s
case test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config: 0.414s
case test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run: 0.118s
case test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail: 0.434s
case test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails: 0.315s
case test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass: 0.437s
case test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: 0.000s
case test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: 0.000s
case test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: 0.000s
case test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: 0.000s
case test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: 0.000s
case test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: 0.000s
case test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: 0.000s
case test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: 0.000s
case test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: 0.000s
case test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: 0.000s
case test_vt_lib.VtLibTests.test_term_while_away_restores_home: 0.000s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: 1.238s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals: 0.004s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge: 0.243s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta: 0.001s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry: 0.215s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light: 0.019s
case test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit: 0.124s
case test_package_pin.PinTest.test_check_rejects_a_hand_edited_count: 0.117s
case test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline: 0.077s
case test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values: 0.076s
case test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: 0.000s
case test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: 0.000s
case test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: 0.000s
case test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: 0.000s
case test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: 0.000s
case test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_export: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: 0.000s
case test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: 0.000s
case test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time: 0.000s
case test_optic_settling.EdgeTests.test_window_counts_half_open_intervals: 0.000s
case test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only: 0.002s
case test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup: 0.005s
case test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed: 0.166s
case test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane: 0.286s
case test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime: 0.506s
case test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case: 0.009s
case test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval: 0.004s
case test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request: 0.010s
case test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked: 0.013s
case test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws: 0.009s
case test_optic_settling.RunTests.test_collect_frame_must_redraw: 0.004s
case test_optic_settling.RunTests.test_complete_lane_passes: 0.011s
case test_optic_settling.RunTests.test_complete_pilot_passes: 0.002s
case test_optic_settling.RunTests.test_consumer_frames_and_samples: 0.006s
case test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale: 0.014s
case test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot: 0.004s
case test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus: 0.005s
case test_optic_settling.RunTests.test_journal_needs_consistent_alignment: 0.003s
case test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases: 0.006s
case test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log: 0.004s
case test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end: 0.006s
case test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls: 0.008s
case test_optic_settling.RunTests.test_rejects_heartbeat_gap: 0.003s
case test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries: 0.010s
case test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases: 0.011s
case test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture: 0.005s
case test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples: 0.007s
case test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace: 0.003s
case test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause: 0.005s
case test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval: 0.003s
case test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window: 0.005s
case test_optic_settling.RunTests.test_trace_without_messages: 0.004s
case test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return: 0.005s
case test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason: 0.003s
case test_optic_settling.RunTests.test_zone_names_with_unquoted_commas: 0.003s
case test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi: 2.624s
case test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry: 0.000s
case test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty: 0.001s
case test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run: 0.000s
case test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run: 0.000s
case test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint: 0.002s
case test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run: 0.018s
case test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process: 0.007s
case test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash: 0.037s
case test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing: 0.003s
case test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source: 0.025s
case test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing: 0.002s
case test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing: 0.013s
case test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash: 0.047s
case test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames: 0.027s
case test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text: 0.043s
case test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses: 0.001s
case test_capture_meta.JudgementTests.test_quiet_and_settle: 0.000s
case test_capture_meta.JudgementTests.test_threshold_validation: 0.000s
case test_capture_meta.LockTests.test_acquire_release_and_ownership_check: 0.001s
case test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder: 0.301s
case test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed: 0.001s
case test_capture_meta.LockTests.test_public_read_waits_for_guard: 0.301s
case test_capture_meta.LockTests.test_rejects_invalid_owner_fields: 0.002s
case test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one: 0.133s
case test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients: 0.005s
case test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run: 0.002s
case test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load: 0.011s
case test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients: 0.017s
case test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu: 0.006s
case test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling: 0.004s
case test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock: 0.005s
case test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run: 0.001s
case test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock: 0.003s
case test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight: 0.003s
case test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load: 0.003s
case test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order: 0.001s
case test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list: 0.000s
case test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run: 0.000s
case test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema: 0.000s
case test_capture_meta.RecordTests.test_load_record_rejects_non_object_json: 0.000s
case test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation: 0.387s
case test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite: 0.001s
case test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks: 0.001s
case test_capture_meta.SamplingTests.test_sample_stream_and_summary: 0.000s
case test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients: 0.000s
case test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample: 0.007s
case test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse: 0.007s
case test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises: 0.007s
case test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock: 0.003s
case test_capture_meta.SettleTests.test_release_command_is_ownership_checked: 0.010s
case test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses: 0.004s
case test_capture_meta.SettleTests.test_settled_entry_carries_inputs: 0.004s
case test_capture_meta.ShowTests.test_main_show_exit_codes: 0.002s
case test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts: 0.000s
case test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary: 0.475s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero: 0.374s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error: 0.390s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error: 0.438s
case test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session: 0.476s
case test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero: 0.381s
case test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged: 0.000s
case test_screencast_consumer.PackedRgbTests.test_short_buffer_raises: 0.000s
case test_screencast_consumer.PackedRgbTests.test_strips_row_padding: 0.000s
case test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl: 0.000s
case test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending: 0.000s
case test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request: 0.000s
case test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending: 0.001s
case test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last: 0.001s
case test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request: 0.001s
case test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption: 0.754s
case test_screencast_consumer.WaitForTests.test_times_out_without_a_node: 0.201s
case test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full: 0.261s
case test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full: 0.272s
case test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate: 0.030s
case test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr: 0.121s
case test_gates.Gates.test_focused_recipe_records_failure_through_host_budget: 0.129s
case test_gates.Gates.test_focused_recipe_requires_arguments: 0.012s
case test_gates.Gates.test_full_recipe_failure_stops_before_following_stages: 0.105s
case test_gates.Gates.test_lfs_failure_stops_before_gate: 0.061s
case test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast: 1.592s
case test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input: 0.350s
case test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target: 1.194s
case test_gates.Gates.test_staged_paths_select_docs_only_conservatively: 0.682s
case test_affected.Select.test_docs_and_tasks_select_nothing: 0.000s
case test_affected.Select.test_excluded_member_is_never_selected: 0.000s
case test_affected.Select.test_member_change_selects_its_dependents: 0.000s
case test_affected.Select.test_nothing_changed_selects_nothing: 0.000s
case test_affected.Select.test_root_source_selects_the_root_package: 0.000s
case test_affected.Select.test_workspace_files_select_everything: 0.000s
case test_vdrag.WireTest.test_fixed_is_24_8: 0.000s
case test_vdrag.WireTest.test_header_packs_object_size_and_opcode: 0.000s
case test_vdrag.WireTest.test_parse_global: 0.000s
case test_vdrag.WireTest.test_split_messages_handles_partial_tail: 0.000s
case test_vdrag.WireTest.test_string_is_padded_and_nul_terminated: 0.000s
case test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied: 0.000s
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_term_while_away_restores_home: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_export: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0

Ran 320 tests in 20.031s
OK (skipped=37)
```

## amend-affected-rust.log

```text
     Summary [   0.000s] 0 tests run: 0 passed, 0 skipped (no workspace package changed against HEAD)
```

## amend-ubuntu-check/pilot.log

```text
test_term_during_journal_end_preserves_exit_and_cleanup (tools.test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup) ... ok
test_term_with_the_session_locked_reaps_the_lock_client (tools.test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client) ... ok
test_startup_sampling_and_summary (tools.test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary) ... ok
test_focused_recipe_preserves_arguments_and_counts_stderr (tools.test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr) ... ok
test_worktrees_with_their_own_target_pass (tools.test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass) ... ok

----------------------------------------------------------------------
Ran 5 tests in 5.005s

OK
```

## amend-ubuntu-check/focus.log

```text
.................................
----------------------------------------------------------------------
Ran 33 tests in 7.684s

OK
```

## amend-ubuntu-check/fast.log

```text
Fast tooling: 320 cases, 2 children
case test_upstream_report.Baseline.test_fork_tree_mismatch_fails: 0.082s
case test_upstream_report.Baseline.test_merge_base_is_never_consulted: 0.081s
case test_upstream_report.Baseline.test_missing_carried_key_fails: 0.068s
case test_upstream_report.Baseline.test_recorded_tree_mismatch_fails: 0.080s
case test_upstream_report.Baseline.test_resolution_needs_no_branches: 0.092s
case test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates: 0.087s
case test_upstream_report.Baseline.test_tag_naming_another_tree_fails: 0.066s
case test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates: 0.110s
case test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail: 0.090s
case test_upstream_report.Classify.test_added_scaffolding_is_class_c: 0.000s
case test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_modified_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_other_additions_are_class_a: 0.000s
case test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one: 0.243s
case test_upstream_report.Cli.test_drift_conflict_exits_one: 0.194s
case test_upstream_report.Cli.test_fresh_report_exits_zero: 0.230s
case test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback: 0.125s
case test_upstream_report.Cli.test_missing_markers_exit_two: 0.164s
case test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits: 0.355s
case test_upstream_report.Cli.test_stale_report_exits_one: 0.161s
case test_upstream_report.Cli.test_truncated_hash_exits_two: 0.125s
case test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one: 0.011s
case test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback: 0.244s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away: 0.067s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one: 0.073s
case test_upstream_report.Conflicts.test_acknowledged_path_passes: 0.015s
case test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result: 0.060s
case test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts: 0.037s
case test_upstream_report.Conflicts.test_clean_status_never_produces_findings: 0.014s
case test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails: 0.010s
case test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths: 0.057s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict: 0.068s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict: 0.063s
case test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries: 0.071s
case test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses: 0.016s
case test_upstream_report.Conflicts.test_malformed_informational_section_raises: 0.015s
case test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding: 0.016s
case test_upstream_report.Conflicts.test_unattributed_conflict_fails: 0.015s
case test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path: 0.016s
case test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag: 0.118s
case test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename: 0.099s
case test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit: 0.105s
case test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name: 0.093s
case test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit: 0.124s
case test_upstream_report.Freshness.test_generation_is_a_fixpoint: 0.107s
case test_upstream_report.Freshness.test_regenerated_and_staged_report_passes: 0.121s
case test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose: 0.101s
case test_upstream_report.Freshness.test_stale_report_is_a_finding: 0.104s
case test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored: 0.139s
case test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict: 0.129s
case test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error: 0.082s
case test_upstream_report.Inventory.test_inventory_excludes_task_records: 0.095s
case test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files: 0.088s
case test_upstream_report.Inventory.test_inventory_is_sorted_by_path: 0.093s
case test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree: 0.085s
case test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts: 0.081s
case test_upstream_report.Inventory.test_local_block_counts_classes: 0.089s
case test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path: 0.076s
case test_upstream_report.Splice.test_markers_out_of_order_are_an_error: 0.000s
case test_upstream_report.Splice.test_missing_markers_are_an_error: 0.000s
case test_upstream_report.Splice.test_splice_is_idempotent: 0.000s
case test_upstream_report.Splice.test_splice_replaces_only_between_markers: 0.000s
case test_upstream_report.Stage.test_fresh_report_is_left_alone: 0.124s
case test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale: 0.129s
case test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged: 0.103s
case test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes: 0.094s
case test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report: 0.113s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal: 0.003s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest: 0.012s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles: 0.585s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls: 0.050s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments: 0.004s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name: 0.005s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub: 0.005s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build: 0.000s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights: 0.012s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch: 0.010s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight: 0.005s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build: 0.003s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs: 0.246s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup: 0.014s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope: 0.007s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order: 0.002s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound: 0.087s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture: 0.003s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach: 0.014s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam: 0.001s
case test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces: 0.030s
case test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail: 0.039s
case test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution: 0.061s
case test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail: 0.449s
case test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting: 0.136s
case test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery: 0.068s
case test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures: 0.323s
case test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery: 0.132s
case test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids: 0.195s
case test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping: 1.292s
case test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case: 0.297s
case test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn: 0.062s
case test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted: 0.148s
case test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure: 0.272s
case test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values: 0.002s
case test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids: 0.000s
case test_tooling_tests.ModeTests.test_modes_fail_early: 0.001s
case test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping: 0.131s
case test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail: 0.036s
case test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers: 0.005s
case test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed: 0.023s
case test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd: 0.007s
case test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed: 0.018s
case test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared: 0.121s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped: 0.188s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped: 0.121s
case test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named: 0.153s
case test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named: 0.154s
case test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped: 0.205s
case test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes: 0.133s
case test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped: 0.204s
case test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory: 0.251s
case test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared: 0.149s
case test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config: 0.323s
case test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run: 0.126s
case test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail: 0.293s
case test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails: 0.215s
case test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass: 0.323s
case test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: 0.000s
case test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: 0.000s
case test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: 0.000s
case test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: 0.000s
case test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: 0.000s
case test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: 0.000s
case test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: 0.000s
case test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: 0.000s
case test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: 0.000s
case test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: 0.000s
case test_vt_lib.VtLibTests.test_term_while_away_restores_home: 0.000s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: 0.000s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals: 0.008s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge: 0.337s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta: 0.001s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry: 0.291s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light: 0.023s
case test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit: 0.104s
case test_package_pin.PinTest.test_check_rejects_a_hand_edited_count: 0.121s
case test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline: 0.061s
case test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values: 0.065s
case test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: 0.000s
case test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: 0.000s
case test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: 0.000s
case test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: 0.000s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: 0.000s
case test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: 0.000s
case test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_during_export: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: 0.000s
case test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: 0.000s
case test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: 0.000s
case test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time: 0.000s
case test_optic_settling.EdgeTests.test_window_counts_half_open_intervals: 0.000s
case test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only: 0.002s
case test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup: 0.005s
case test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed: 0.169s
case test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane: 0.263s
case test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime: 0.386s
case test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case: 0.010s
case test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval: 0.005s
case test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request: 0.011s
case test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked: 0.015s
case test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws: 0.007s
case test_optic_settling.RunTests.test_collect_frame_must_redraw: 0.003s
case test_optic_settling.RunTests.test_complete_lane_passes: 0.012s
case test_optic_settling.RunTests.test_complete_pilot_passes: 0.003s
case test_optic_settling.RunTests.test_consumer_frames_and_samples: 0.006s
case test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale: 0.015s
case test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot: 0.004s
case test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus: 0.005s
case test_optic_settling.RunTests.test_journal_needs_consistent_alignment: 0.003s
case test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases: 0.006s
case test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log: 0.004s
case test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end: 0.006s
case test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls: 0.010s
case test_optic_settling.RunTests.test_rejects_heartbeat_gap: 0.003s
case test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries: 0.011s
case test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases: 0.012s
case test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture: 0.005s
case test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples: 0.007s
case test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace: 0.003s
case test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause: 0.005s
case test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval: 0.003s
case test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window: 0.005s
case test_optic_settling.RunTests.test_trace_without_messages: 0.004s
case test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return: 0.006s
case test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason: 0.003s
case test_optic_settling.RunTests.test_zone_names_with_unquoted_commas: 0.003s
case test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi: 2.504s
case test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry: 0.001s
case test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty: 0.001s
case test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run: 0.000s
case test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run: 0.000s
case test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint: 0.001s
case test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run: 0.006s
case test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process: 0.002s
case test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash: 0.034s
case test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing: 0.003s
case test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source: 0.024s
case test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing: 0.003s
case test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing: 0.014s
case test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash: 0.036s
case test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames: 0.023s
case test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text: 0.037s
case test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses: 0.001s
case test_capture_meta.JudgementTests.test_quiet_and_settle: 0.001s
case test_capture_meta.JudgementTests.test_threshold_validation: 0.000s
case test_capture_meta.LockTests.test_acquire_release_and_ownership_check: 0.002s
case test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder: 0.302s
case test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed: 0.001s
case test_capture_meta.LockTests.test_public_read_waits_for_guard: 0.301s
case test_capture_meta.LockTests.test_rejects_invalid_owner_fields: 0.002s
case test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one: 0.080s
case test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients: 0.004s
case test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run: 0.001s
case test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load: 0.007s
case test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients: 0.008s
case test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu: 0.003s
case test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling: 0.001s
case test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock: 0.003s
case test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run: 0.001s
case test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock: 0.002s
case test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight: 0.003s
case test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load: 0.003s
case test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order: 0.001s
case test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list: 0.000s
case test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run: 0.000s
case test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema: 0.001s
case test_capture_meta.RecordTests.test_load_record_rejects_non_object_json: 0.000s
case test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation: 0.316s
case test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite: 0.001s
case test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks: 0.001s
case test_capture_meta.SamplingTests.test_sample_stream_and_summary: 0.000s
case test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients: 0.000s
case test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample: 0.006s
case test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse: 0.008s
case test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises: 0.006s
case test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock: 0.004s
case test_capture_meta.SettleTests.test_release_command_is_ownership_checked: 0.009s
case test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses: 0.005s
case test_capture_meta.SettleTests.test_settled_entry_carries_inputs: 0.006s
case test_capture_meta.ShowTests.test_main_show_exit_codes: 0.004s
case test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts: 0.000s
case test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary: 0.348s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero: 0.232s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error: 0.239s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error: 0.311s
case test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session: 0.262s
case test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero: 0.232s
case test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged: 0.000s
case test_screencast_consumer.PackedRgbTests.test_short_buffer_raises: 0.000s
case test_screencast_consumer.PackedRgbTests.test_strips_row_padding: 0.000s
case test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl: 0.000s
case test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending: 0.001s
case test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request: 0.000s
case test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending: 0.001s
case test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last: 0.002s
case test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request: 0.001s
case test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption: 0.752s
case test_screencast_consumer.WaitForTests.test_times_out_without_a_node: 0.201s
case test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full: 0.162s
case test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full: 0.170s
case test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate: 0.017s
case test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr: 0.098s
case test_gates.Gates.test_focused_recipe_records_failure_through_host_budget: 0.087s
case test_gates.Gates.test_focused_recipe_requires_arguments: 0.014s
case test_gates.Gates.test_full_recipe_failure_stops_before_following_stages: 0.090s
case test_gates.Gates.test_lfs_failure_stops_before_gate: 0.043s
case test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast: 1.185s
case test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input: 0.226s
case test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target: 1.007s
case test_gates.Gates.test_staged_paths_select_docs_only_conservatively: 0.550s
case test_affected.Select.test_docs_and_tasks_select_nothing: 0.000s
case test_affected.Select.test_excluded_member_is_never_selected: 0.000s
case test_affected.Select.test_member_change_selects_its_dependents: 0.000s
case test_affected.Select.test_nothing_changed_selects_nothing: 0.000s
case test_affected.Select.test_root_source_selects_the_root_package: 0.000s
case test_affected.Select.test_workspace_files_select_everything: 0.000s
case test_vdrag.WireTest.test_fixed_is_24_8: 0.000s
case test_vdrag.WireTest.test_header_packs_object_size_and_opcode: 0.000s
case test_vdrag.WireTest.test_parse_global: 0.000s
case test_vdrag.WireTest.test_split_messages_handles_partial_tail: 0.000s
case test_vdrag.WireTest.test_string_is_padded_and_nul_terminated: 0.000s
case test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied: 0.000s
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: ImageMagick is required for positive face probes
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: ImageMagick is required for pixel probes
SKIP test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_vt_lib.VtLibTests.test_term_while_away_restores_home: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: ImageMagick is required for capture decoding
SKIP test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_during_export: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0
SKIP test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0

Ran 320 tests in 14.660s
OK (skipped=40)
```

## amend-ubuntu-check/full.log

```text
Full tooling: 320 cases, 10 children
case test_affected.Select.test_docs_and_tasks_select_nothing: 0.000s
case test_affected.Select.test_excluded_member_is_never_selected: 0.000s
case test_affected.Select.test_member_change_selects_its_dependents: 0.000s
case test_affected.Select.test_nothing_changed_selects_nothing: 0.000s
case test_affected.Select.test_root_source_selects_the_root_package: 0.000s
case test_affected.Select.test_workspace_files_select_everything: 0.000s
case test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi: 2.789s
case test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry: 0.001s
case test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty: 0.001s
case test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run: 0.000s
case test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run: 0.000s
case test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint: 0.001s
case test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run: 0.006s
case test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process: 0.002s
case test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash: 0.036s
case test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing: 0.004s
case test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source: 0.025s
case test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing: 0.002s
case test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing: 0.014s
case test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash: 0.039s
case test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames: 0.021s
case test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text: 0.041s
case test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses: 0.001s
case test_capture_meta.JudgementTests.test_quiet_and_settle: 0.001s
case test_capture_meta.JudgementTests.test_threshold_validation: 0.000s
case test_capture_meta.LockTests.test_acquire_release_and_ownership_check: 0.002s
case test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder: 0.301s
case test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed: 0.001s
case test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed: 0.001s
case test_capture_meta.LockTests.test_public_read_waits_for_guard: 0.301s
case test_capture_meta.LockTests.test_rejects_invalid_owner_fields: 0.002s
case test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one: 0.079s
case test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients: 0.003s
case test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run: 0.001s
case test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load: 0.006s
case test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients: 0.012s
case test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu: 0.005s
case test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling: 0.002s
case test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock: 0.004s
case test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run: 0.002s
case test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock: 0.004s
case test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight: 0.005s
case test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load: 0.005s
case test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order: 0.001s
case test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list: 0.001s
case test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run: 0.001s
case test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema: 0.001s
case test_capture_meta.RecordTests.test_load_record_rejects_non_object_json: 0.000s
case test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation: 0.318s
case test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite: 0.001s
case test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks: 0.001s
case test_capture_meta.SamplingTests.test_sample_stream_and_summary: 0.000s
case test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients: 0.000s
case test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample: 0.005s
case test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse: 0.005s
case test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises: 0.006s
case test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock: 0.004s
case test_capture_meta.SettleTests.test_release_command_is_ownership_checked: 0.009s
case test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses: 0.005s
case test_capture_meta.SettleTests.test_settled_entry_carries_inputs: 0.006s
case test_capture_meta.ShowTests.test_main_show_exit_codes: 0.004s
case test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts: 0.000s
case test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full: 0.135s
case test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full: 0.175s
case test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate: 0.015s
case test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr: 0.109s
case test_gates.Gates.test_focused_recipe_records_failure_through_host_budget: 0.094s
case test_gates.Gates.test_focused_recipe_requires_arguments: 0.013s
case test_gates.Gates.test_full_recipe_failure_stops_before_following_stages: 0.086s
case test_gates.Gates.test_lfs_failure_stops_before_gate: 0.044s
case test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast: 1.139s
case test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input: 0.209s
case test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target: 1.026s
case test_gates.Gates.test_staged_paths_select_docs_only_conservatively: 0.466s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal: 0.002s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest: 0.012s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles: 0.613s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls: 0.045s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments: 0.003s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name: 0.005s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub: 0.006s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build: 0.001s
case test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights: 0.012s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float: 0.004s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch: 0.012s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight: 0.005s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build: 0.003s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median: 0.005s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace: 0.006s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry: 0.001s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs: 0.237s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup: 0.013s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope: 0.005s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order: 0.002s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound: 0.072s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate: 0.000s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture: 0.002s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach: 0.010s
case test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam: 0.000s
case test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces: 0.026s
case test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail: 0.037s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: 0.000s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals: 0.005s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge: 0.341s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta: 0.001s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry: 0.305s
case test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light: 0.022s
case test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time: 0.000s
case test_optic_settling.EdgeTests.test_window_counts_half_open_intervals: 0.000s
case test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only: 0.002s
case test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup: 0.005s
case test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed: 0.177s
case test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane: 0.254s
case test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime: 0.382s
case test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case: 0.010s
case test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval: 0.005s
case test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request: 0.007s
case test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked: 0.009s
case test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws: 0.007s
case test_optic_settling.RunTests.test_collect_frame_must_redraw: 0.006s
case test_optic_settling.RunTests.test_complete_lane_passes: 0.013s
case test_optic_settling.RunTests.test_complete_pilot_passes: 0.003s
case test_optic_settling.RunTests.test_consumer_frames_and_samples: 0.006s
case test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale: 0.017s
case test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot: 0.004s
case test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus: 0.005s
case test_optic_settling.RunTests.test_journal_needs_consistent_alignment: 0.003s
case test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases: 0.007s
case test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log: 0.004s
case test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end: 0.007s
case test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls: 0.009s
case test_optic_settling.RunTests.test_rejects_heartbeat_gap: 0.003s
case test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries: 0.013s
case test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases: 0.013s
case test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture: 0.006s
case test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples: 0.009s
case test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace: 0.003s
case test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause: 0.006s
case test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval: 0.004s
case test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window: 0.007s
case test_optic_settling.RunTests.test_trace_without_messages: 0.006s
case test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return: 0.008s
case test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason: 0.004s
case test_optic_settling.RunTests.test_zone_names_with_unquoted_commas: 0.004s
case test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit: 0.100s
case test_package_pin.PinTest.test_check_rejects_a_hand_edited_count: 0.094s
case test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline: 0.058s
case test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values: 0.050s
case test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary: 0.333s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero: 0.217s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error: 0.236s
case test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error: 0.304s
case test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session: 0.275s
case test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero: 0.248s
case test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged: 0.000s
case test_screencast_consumer.PackedRgbTests.test_short_buffer_raises: 0.000s
case test_screencast_consumer.PackedRgbTests.test_strips_row_padding: 0.000s
case test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl: 0.000s
case test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending: 0.000s
case test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request: 0.000s
case test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending: 0.000s
case test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last: 0.001s
case test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request: 0.001s
case test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption: 0.752s
case test_screencast_consumer.WaitForTests.test_times_out_without_a_node: 0.201s
case test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared: 0.138s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped: 0.190s
case test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped: 0.122s
case test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named: 0.156s
case test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named: 0.130s
case test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped: 0.189s
case test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes: 0.130s
case test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped: 0.173s
case test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory: 0.217s
case test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared: 0.128s
case test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config: 0.320s
case test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run: 0.110s
case test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail: 0.282s
case test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails: 0.217s
case test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass: 0.283s
case test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution: 0.061s
case test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail: 0.453s
case test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting: 0.144s
case test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery: 0.060s
case test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures: 0.333s
case test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery: 0.135s
case test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids: 0.195s
case test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping: 1.298s
case test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case: 0.317s
case test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn: 0.070s
case test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted: 0.123s
case test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure: 0.242s
case test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values: 0.002s
case test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids: 0.000s
case test_tooling_tests.ModeTests.test_modes_fail_early: 0.001s
case test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping: 0.127s
case test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail: 0.091s
case test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers: 0.003s
case test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed: 0.019s
case test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd: 0.010s
case test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed: 0.019s
case test_upstream_report.Baseline.test_fork_tree_mismatch_fails: 0.295s
case test_upstream_report.Baseline.test_merge_base_is_never_consulted: 0.103s
case test_upstream_report.Baseline.test_missing_carried_key_fails: 0.064s
case test_upstream_report.Baseline.test_recorded_tree_mismatch_fails: 0.080s
case test_upstream_report.Baseline.test_resolution_needs_no_branches: 0.105s
case test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates: 0.086s
case test_upstream_report.Baseline.test_tag_naming_another_tree_fails: 0.070s
case test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates: 0.110s
case test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail: 0.082s
case test_upstream_report.Classify.test_added_scaffolding_is_class_c: 0.000s
case test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_modified_upstream_files_are_class_b: 0.000s
case test_upstream_report.Classify.test_other_additions_are_class_a: 0.000s
case test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one: 0.229s
case test_upstream_report.Cli.test_drift_conflict_exits_one: 0.186s
case test_upstream_report.Cli.test_fresh_report_exits_zero: 0.229s
case test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback: 0.142s
case test_upstream_report.Cli.test_missing_markers_exit_two: 0.160s
case test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits: 0.372s
case test_upstream_report.Cli.test_stale_report_exits_one: 0.154s
case test_upstream_report.Cli.test_truncated_hash_exits_two: 0.138s
case test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one: 0.016s
case test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback: 0.267s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away: 0.069s
case test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one: 0.065s
case test_upstream_report.Conflicts.test_acknowledged_path_passes: 0.010s
case test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result: 0.053s
case test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts: 0.037s
case test_upstream_report.Conflicts.test_clean_status_never_produces_findings: 0.013s
case test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails: 0.016s
case test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths: 0.055s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict: 0.074s
case test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict: 0.068s
case test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries: 0.071s
case test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses: 0.016s
case test_upstream_report.Conflicts.test_malformed_informational_section_raises: 0.015s
case test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding: 0.015s
case test_upstream_report.Conflicts.test_unattributed_conflict_fails: 0.016s
case test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path: 0.015s
case test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag: 0.116s
case test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename: 0.110s
case test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit: 0.104s
case test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name: 0.092s
case test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit: 0.130s
case test_upstream_report.Freshness.test_generation_is_a_fixpoint: 0.110s
case test_upstream_report.Freshness.test_regenerated_and_staged_report_passes: 0.120s
case test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose: 0.108s
case test_upstream_report.Freshness.test_stale_report_is_a_finding: 0.097s
case test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored: 0.128s
case test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict: 0.091s
case test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error: 0.063s
case test_upstream_report.Inventory.test_inventory_excludes_task_records: 0.090s
case test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files: 0.090s
case test_upstream_report.Inventory.test_inventory_is_sorted_by_path: 0.095s
case test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree: 0.099s
case test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts: 0.085s
case test_upstream_report.Inventory.test_local_block_counts_classes: 0.110s
case test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path: 0.084s
case test_upstream_report.Splice.test_markers_out_of_order_are_an_error: 0.000s
case test_upstream_report.Splice.test_missing_markers_are_an_error: 0.000s
case test_upstream_report.Splice.test_splice_is_idempotent: 0.000s
case test_upstream_report.Splice.test_splice_replaces_only_between_markers: 0.000s
case test_upstream_report.Stage.test_fresh_report_is_left_alone: 0.116s
case test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale: 0.131s
case test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged: 0.127s
case test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes: 0.105s
case test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report: 0.118s
case test_vdrag.WireTest.test_fixed_is_24_8: 0.000s
case test_vdrag.WireTest.test_header_packs_object_size_and_opcode: 0.000s
case test_vdrag.WireTest.test_parse_global: 0.000s
case test_vdrag.WireTest.test_split_messages_handles_partial_tail: 0.000s
case test_vdrag.WireTest.test_string_is_padded_and_nul_terminated: 0.000s
case test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied: 0.000s
case test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end: 17.753s
case test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused: 0.135s
case test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound: 15.167s
case test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release: 10.386s
case test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops: 14.879s
case test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch: 0.168s
case test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases: 5.031s
case test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart: 7.089s
case test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch: 0.333s
case test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item: 0.155s
case test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record: 0.095s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer: 10.731s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe: 8.440s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size: 8.449s
case test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size: 11.743s
case test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools: 0.172s
case test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record: 0.173s
case test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt: 3.222s
case test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait: 3.228s
case test_optic_settling.DriverCleanupTests.test_term_during_export: 14.959s
case test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus: 10.700s
case test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release: 4.080s
case test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client: 4.069s
case test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools: 0.275s
case test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded: 8.020s
case test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt: 6.275s
case test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json: 6.272s
case test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched: 0.007s
case test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores: 0.021s
case test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails: 0.014s
case test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions: 0.012s
case test_vt_lib.VtLibTests.test_spare_skips_the_home_vt: 0.013s
case test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails: 0.041s
case test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed: 2.097s
case test_vt_lib.VtLibTests.test_term_while_away_restores_home: 0.064s
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code: ImageMagick is required for positive face probes
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate: set MATERIAL_RETAINED_NIRI to validate generated KDL
SKIP test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output: ImageMagick is required for pixel probes
SKIP test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture: ImageMagick is required for capture decoding

Ran 320 tests in 26.069s
OK (skipped=5)
```

## amend-native-host.json

```json
{
  "ids": [
    "test_affected.Select.test_docs_and_tasks_select_nothing",
    "test_affected.Select.test_excluded_member_is_never_selected",
    "test_affected.Select.test_member_change_selects_its_dependents",
    "test_affected.Select.test_nothing_changed_selects_nothing",
    "test_affected.Select.test_root_source_selects_the_root_package",
    "test_affected.Select.test_workspace_files_select_everything",
    "test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi",
    "test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry",
    "test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty",
    "test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run",
    "test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run",
    "test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint",
    "test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run",
    "test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process",
    "test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash",
    "test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing",
    "test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source",
    "test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing",
    "test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing",
    "test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash",
    "test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames",
    "test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text",
    "test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses",
    "test_capture_meta.JudgementTests.test_quiet_and_settle",
    "test_capture_meta.JudgementTests.test_threshold_validation",
    "test_capture_meta.LockTests.test_acquire_release_and_ownership_check",
    "test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed",
    "test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder",
    "test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed",
    "test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed",
    "test_capture_meta.LockTests.test_public_read_waits_for_guard",
    "test_capture_meta.LockTests.test_rejects_invalid_owner_fields",
    "test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one",
    "test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients",
    "test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run",
    "test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load",
    "test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients",
    "test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu",
    "test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling",
    "test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock",
    "test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run",
    "test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock",
    "test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight",
    "test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load",
    "test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order",
    "test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list",
    "test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run",
    "test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema",
    "test_capture_meta.RecordTests.test_load_record_rejects_non_object_json",
    "test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation",
    "test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite",
    "test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks",
    "test_capture_meta.SamplingTests.test_sample_stream_and_summary",
    "test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients",
    "test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample",
    "test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse",
    "test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises",
    "test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock",
    "test_capture_meta.SettleTests.test_release_command_is_ownership_checked",
    "test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses",
    "test_capture_meta.SettleTests.test_settled_entry_carries_inputs",
    "test_capture_meta.ShowTests.test_main_show_exit_codes",
    "test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts",
    "test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full",
    "test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full",
    "test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate",
    "test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr",
    "test_gates.Gates.test_focused_recipe_records_failure_through_host_budget",
    "test_gates.Gates.test_focused_recipe_requires_arguments",
    "test_gates.Gates.test_full_recipe_failure_stops_before_following_stages",
    "test_gates.Gates.test_lfs_failure_stops_before_gate",
    "test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast",
    "test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input",
    "test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target",
    "test_gates.Gates.test_staged_paths_select_docs_only_conservatively",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam",
    "test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces",
    "test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light",
    "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end",
    "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused",
    "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound",
    "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release",
    "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops",
    "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch",
    "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases",
    "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart",
    "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch",
    "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item",
    "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size",
    "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools",
    "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record",
    "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt",
    "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait",
    "test_optic_settling.DriverCleanupTests.test_term_during_export",
    "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus",
    "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release",
    "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client",
    "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools",
    "test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time",
    "test_optic_settling.EdgeTests.test_window_counts_half_open_intervals",
    "test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only",
    "test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup",
    "test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed",
    "test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane",
    "test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime",
    "test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case",
    "test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval",
    "test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request",
    "test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked",
    "test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws",
    "test_optic_settling.RunTests.test_collect_frame_must_redraw",
    "test_optic_settling.RunTests.test_complete_lane_passes",
    "test_optic_settling.RunTests.test_complete_pilot_passes",
    "test_optic_settling.RunTests.test_consumer_frames_and_samples",
    "test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale",
    "test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot",
    "test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus",
    "test_optic_settling.RunTests.test_journal_needs_consistent_alignment",
    "test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases",
    "test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log",
    "test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end",
    "test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls",
    "test_optic_settling.RunTests.test_rejects_heartbeat_gap",
    "test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries",
    "test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases",
    "test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture",
    "test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples",
    "test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace",
    "test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause",
    "test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval",
    "test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window",
    "test_optic_settling.RunTests.test_trace_without_messages",
    "test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return",
    "test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason",
    "test_optic_settling.RunTests.test_zone_names_with_unquoted_commas",
    "test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit",
    "test_package_pin.PinTest.test_check_rejects_a_hand_edited_count",
    "test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline",
    "test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values",
    "test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error",
    "test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session",
    "test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero",
    "test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged",
    "test_screencast_consumer.PackedRgbTests.test_short_buffer_raises",
    "test_screencast_consumer.PackedRgbTests.test_strips_row_padding",
    "test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl",
    "test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending",
    "test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request",
    "test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending",
    "test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last",
    "test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request",
    "test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption",
    "test_screencast_consumer.WaitForTests.test_times_out_without_a_node",
    "test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared",
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named",
    "test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named",
    "test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes",
    "test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory",
    "test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared",
    "test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config",
    "test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run",
    "test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail",
    "test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails",
    "test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass",
    "test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution",
    "test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail",
    "test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting",
    "test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery",
    "test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures",
    "test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery",
    "test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids",
    "test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping",
    "test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case",
    "test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn",
    "test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted",
    "test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure",
    "test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values",
    "test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids",
    "test_tooling_tests.ModeTests.test_modes_fail_early",
    "test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping",
    "test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail",
    "test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers",
    "test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed",
    "test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd",
    "test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed",
    "test_upstream_report.Baseline.test_fork_tree_mismatch_fails",
    "test_upstream_report.Baseline.test_merge_base_is_never_consulted",
    "test_upstream_report.Baseline.test_missing_carried_key_fails",
    "test_upstream_report.Baseline.test_recorded_tree_mismatch_fails",
    "test_upstream_report.Baseline.test_resolution_needs_no_branches",
    "test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates",
    "test_upstream_report.Baseline.test_tag_naming_another_tree_fails",
    "test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates",
    "test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail",
    "test_upstream_report.Classify.test_added_scaffolding_is_class_c",
    "test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b",
    "test_upstream_report.Classify.test_modified_upstream_files_are_class_b",
    "test_upstream_report.Classify.test_other_additions_are_class_a",
    "test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one",
    "test_upstream_report.Cli.test_drift_conflict_exits_one",
    "test_upstream_report.Cli.test_fresh_report_exits_zero",
    "test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback",
    "test_upstream_report.Cli.test_missing_markers_exit_two",
    "test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits",
    "test_upstream_report.Cli.test_stale_report_exits_one",
    "test_upstream_report.Cli.test_truncated_hash_exits_two",
    "test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one",
    "test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback",
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away",
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one",
    "test_upstream_report.Conflicts.test_acknowledged_path_passes",
    "test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result",
    "test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts",
    "test_upstream_report.Conflicts.test_clean_status_never_produces_findings",
    "test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails",
    "test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths",
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict",
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict",
    "test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries",
    "test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses",
    "test_upstream_report.Conflicts.test_malformed_informational_section_raises",
    "test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding",
    "test_upstream_report.Conflicts.test_unattributed_conflict_fails",
    "test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path",
    "test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag",
    "test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename",
    "test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit",
    "test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name",
    "test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit",
    "test_upstream_report.Freshness.test_generation_is_a_fixpoint",
    "test_upstream_report.Freshness.test_regenerated_and_staged_report_passes",
    "test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose",
    "test_upstream_report.Freshness.test_stale_report_is_a_finding",
    "test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored",
    "test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict",
    "test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error",
    "test_upstream_report.Inventory.test_inventory_excludes_task_records",
    "test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files",
    "test_upstream_report.Inventory.test_inventory_is_sorted_by_path",
    "test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree",
    "test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts",
    "test_upstream_report.Inventory.test_local_block_counts_classes",
    "test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path",
    "test_upstream_report.Splice.test_markers_out_of_order_are_an_error",
    "test_upstream_report.Splice.test_missing_markers_are_an_error",
    "test_upstream_report.Splice.test_splice_is_idempotent",
    "test_upstream_report.Splice.test_splice_replaces_only_between_markers",
    "test_upstream_report.Stage.test_fresh_report_is_left_alone",
    "test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale",
    "test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged",
    "test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes",
    "test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report",
    "test_vdrag.WireTest.test_fixed_is_24_8",
    "test_vdrag.WireTest.test_header_packs_object_size_and_opcode",
    "test_vdrag.WireTest.test_parse_global",
    "test_vdrag.WireTest.test_split_messages_handles_partial_tail",
    "test_vdrag.WireTest.test_string_is_padded_and_nul_terminated",
    "test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied",
    "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded",
    "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt",
    "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json",
    "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched",
    "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores",
    "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails",
    "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions",
    "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt",
    "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails",
    "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed",
    "test_vt_lib.VtLibTests.test_term_while_away_restores_home"
  ],
  "ran": 320,
  "skipped": [
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_export",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_term_while_away_restores_home",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ]
  ],
  "failures": [],
  "errors": [],
  "expected_failures": [],
  "unexpected_successes": [],
  "elapsed": {
    "test_affected.Select.test_docs_and_tasks_select_nothing": 0.00017607500194571912,
    "test_affected.Select.test_excluded_member_is_never_selected": 6.95429916959256e-05,
    "test_affected.Select.test_member_change_selects_its_dependents": 7.987300341483206e-05,
    "test_affected.Select.test_nothing_changed_selects_nothing": 2.2642998374067247e-05,
    "test_affected.Select.test_root_source_selects_the_root_package": 9.274699550587684e-05,
    "test_affected.Select.test_workspace_files_select_everything": 5.697800952475518e-05,
    "test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi": 2.6290413009992335,
    "test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry": 0.0003873890091199428,
    "test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty": 0.00041716500709299,
    "test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run": 0.00013825400674249977,
    "test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run": 0.00019183600670658052,
    "test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint": 0.001286352999159135,
    "test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run": 0.019734162007807754,
    "test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process": 0.006348256007186137,
    "test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash": 0.047690482999314554,
    "test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing": 0.00802391099568922,
    "test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source": 0.027055023005232215,
    "test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing": 0.002416557996184565,
    "test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing": 0.01536065200343728,
    "test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash": 0.04401913199399132,
    "test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames": 0.025067902999580838,
    "test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text": 0.04737816800479777,
    "test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses": 0.001312573003815487,
    "test_capture_meta.JudgementTests.test_quiet_and_settle": 0.0004970779991708696,
    "test_capture_meta.JudgementTests.test_threshold_validation": 0.00011639200965873897,
    "test_capture_meta.LockTests.test_acquire_release_and_ownership_check": 0.0013758540007984266,
    "test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed": 0.0008856389904394746,
    "test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder": 0.3017673360009212,
    "test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed": 0.005914890003623441,
    "test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed": 0.0012551640102174133,
    "test_capture_meta.LockTests.test_public_read_waits_for_guard": 0.3015608220011927,
    "test_capture_meta.LockTests.test_rejects_invalid_owner_fields": 0.0072248170035891235,
    "test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one": 0.09401781699853018,
    "test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients": 0.007283208004082553,
    "test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run": 0.002438338997308165,
    "test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load": 0.012298472996917553,
    "test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients": 0.016409582996857353,
    "test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu": 0.005703335991711356,
    "test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling": 0.0034999939962290227,
    "test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock": 0.00510396299068816,
    "test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run": 0.0009785459988052025,
    "test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock": 0.0030815259960945696,
    "test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight": 0.0035621129936771467,
    "test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load": 0.0032160920090973377,
    "test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order": 0.0005036200018366799,
    "test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list": 0.00032391800777986646,
    "test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run": 0.00024023799051064998,
    "test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema": 0.0004351899988250807,
    "test_capture_meta.RecordTests.test_load_record_rejects_non_object_json": 0.00026607800100464374,
    "test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation": 0.3465010700019775,
    "test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite": 0.0009015789983095601,
    "test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks": 0.006146361003629863,
    "test_capture_meta.SamplingTests.test_sample_stream_and_summary": 0.00016826100181788206,
    "test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients": 0.00014152999210637063,
    "test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample": 0.006402910003089346,
    "test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse": 0.006770220003090799,
    "test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises": 0.00706355000147596,
    "test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock": 0.0034431949898134917,
    "test_capture_meta.SettleTests.test_release_command_is_ownership_checked": 0.010485624996363185,
    "test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses": 0.006293622005614452,
    "test_capture_meta.SettleTests.test_settled_entry_carries_inputs": 0.006900478998431936,
    "test_capture_meta.ShowTests.test_main_show_exit_codes": 0.0027981460007140413,
    "test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts": 5.5445998441427946e-05,
    "test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full": 0.2423853060026886,
    "test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full": 0.24288417799107265,
    "test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate": 0.028405817996826954,
    "test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr": 0.11727947600593325,
    "test_gates.Gates.test_focused_recipe_records_failure_through_host_budget": 0.10853313699772116,
    "test_gates.Gates.test_focused_recipe_requires_arguments": 0.010964408007566817,
    "test_gates.Gates.test_full_recipe_failure_stops_before_following_stages": 0.09551520200329833,
    "test_gates.Gates.test_lfs_failure_stops_before_gate": 0.059635090990923345,
    "test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast": 1.5205913160025375,
    "test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input": 0.36318187799770385,
    "test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target": 1.211545477999607,
    "test_gates.Gates.test_staged_paths_select_docs_only_conservatively": 0.6715347080025822,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal": 0.002756526999291964,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest": 0.01292228201054968,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles": 1.1685845910105854,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls": 0.09058577100222465,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments": 0.006716538002365269,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name": 0.00732456699188333,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub": 0.00822634699579794,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build": 0.000638106997939758,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights": 0.0007696670072618872,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics": 0.0004087200068170205,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights": 0.015695260008214973,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float": 0.008358368999324739,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch": 0.015130583007703535,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight": 0.006279253997490741,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build": 0.003918362999684177,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait": 0.008191980989067815,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median": 0.006269614998018369,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values": 0.00017198800924234092,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace": 0.007693521009059623,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column": 0.000679094999213703,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry": 0.000812810001662001,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate": 1.2052987585775554e-05,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values": 0.0002245579962618649,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs": 0.3485582089924719,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code": 0.05871977499919012,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate": 1.1732001439668238e-05,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup": 0.021731030006776564,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope": 0.009050909997313283,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output": 0.5704459450062132,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order": 0.003547444997821003,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound": 0.10250917899247725,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate": 0.0002920870028901845,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture": 0.00355062099697534,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach": 0.02205607999349013,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam": 0.0005590859946096316,
    "test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces": 0.05006363800202962,
    "test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail": 0.05576444900361821,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture": 1.2897786650137277,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals": 0.006747375999111682,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge": 0.2636132430052385,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta": 0.0007703189912717789,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry": 0.22035370000230614,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light": 0.020022992001031525,
    "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end": 6.342001142911613e-06,
    "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused": 2.8049980755895376e-06,
    "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound": 2.535001840442419e-06,
    "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release": 2.544999006204307e-06,
    "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops": 2.244007191620767e-06,
    "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch": 2.725006197579205e-06,
    "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases": 2.7049973141402006e-06,
    "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart": 2.415006747469306e-06,
    "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch": 2.0839943317696452e-06,
    "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item": 2.554996171966195e-06,
    "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record": 2.1939922589808702e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer": 2.4750042939558625e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe": 2.3650063667446375e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size": 2.0239967852830887e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size": 2.044005668722093e-06,
    "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools": 2.2939930204302073e-06,
    "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record": 1.9239960238337517e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt": 2.0040024537593126e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait": 2.554996171966195e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_export": 2.0839943317696452e-06,
    "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus": 2.244996721856296e-06,
    "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release": 2.054002834483981e-06,
    "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client": 2.064000000245869e-06,
    "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools": 2.5550107238814235e-06,
    "test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time": 0.00015257099585141987,
    "test_optic_settling.EdgeTests.test_window_counts_half_open_intervals": 5.5665994295850396e-05,
    "test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only": 0.006695607997244224,
    "test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup": 0.005751568009145558,
    "test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed": 0.15704797999933362,
    "test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane": 0.28707497200230137,
    "test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime": 0.5109649049991276,
    "test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case": 0.00852031700196676,
    "test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval": 0.004306681992602535,
    "test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request": 0.00948368500394281,
    "test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked": 0.013479895002092235,
    "test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws": 0.007705184005317278,
    "test_optic_settling.RunTests.test_collect_frame_must_redraw": 0.0029725390049861744,
    "test_optic_settling.RunTests.test_complete_lane_passes": 0.010351469987654127,
    "test_optic_settling.RunTests.test_complete_pilot_passes": 0.0023097439989214763,
    "test_optic_settling.RunTests.test_consumer_frames_and_samples": 0.005150150987901725,
    "test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale": 0.015533701996901073,
    "test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot": 0.003705274997628294,
    "test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus": 0.004641250998247415,
    "test_optic_settling.RunTests.test_journal_needs_consistent_alignment": 0.002109131994075142,
    "test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases": 0.005165360009414144,
    "test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log": 0.003319248993648216,
    "test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end": 0.005474468998727389,
    "test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls": 0.007202243999927305,
    "test_optic_settling.RunTests.test_rejects_heartbeat_gap": 0.002532558995881118,
    "test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries": 0.011388808008632623,
    "test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases": 0.010579715002677403,
    "test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture": 0.004578571009915322,
    "test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples": 0.005846719999681227,
    "test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace": 0.0022044040088076144,
    "test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause": 0.004060012011905201,
    "test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval": 0.002382453007157892,
    "test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window": 0.004283597998437472,
    "test_optic_settling.RunTests.test_trace_without_messages": 0.003362973002367653,
    "test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return": 0.004713237009127624,
    "test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason": 0.002390619003563188,
    "test_optic_settling.RunTests.test_zone_names_with_unquoted_commas": 0.0023424870014423504,
    "test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit": 0.14089654000417795,
    "test_package_pin.PinTest.test_check_rejects_a_hand_edited_count": 0.1497863620024873,
    "test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline": 0.0785905569937313,
    "test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values": 0.07292429199151229,
    "test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary": 0.46204161099740304,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero": 0.35723533300915733,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error": 0.3553387260035379,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error": 0.4225339319964405,
    "test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session": 0.45047036500182003,
    "test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero": 0.36938594198727515,
    "test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged": 6.207800470292568e-05,
    "test_screencast_consumer.PackedRgbTests.test_short_buffer_raises": 7.674699008930475e-05,
    "test_screencast_consumer.PackedRgbTests.test_strips_row_padding": 4.2359999497421086e-05,
    "test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl": 4.2880987166427076e-05,
    "test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending": 0.00039997299609240144,
    "test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request": 0.0002613280084915459,
    "test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending": 0.0004365929926279932,
    "test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last": 0.0015602659987052903,
    "test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request": 0.0009538290032651275,
    "test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption": 0.7549348650063621,
    "test_screencast_consumer.WaitForTests.test_times_out_without_a_node": 0.20127296200371347,
    "test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared": 0.20770311400701758,
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped": 0.2846548149973387,
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped": 0.18200390800484456,
    "test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named": 0.2099561100039864,
    "test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named": 0.1750751249928726,
    "test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped": 0.27615469599550124,
    "test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes": 0.18435736499668565,
    "test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped": 0.24610411199682858,
    "test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory": 0.31955701000697445,
    "test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared": 0.20904888000222854,
    "test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config": 0.47530801499669906,
    "test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run": 0.145276671013562,
    "test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail": 0.4661557810031809,
    "test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails": 0.31910742199397646,
    "test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass": 0.45453708499553613,
    "test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution": 0.09000710600230377,
    "test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail": 0.5445810920064105,
    "test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting": 0.1589358669880312,
    "test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery": 0.07515856099780649,
    "test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures": 0.4014315469976282,
    "test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery": 0.16120668700023089,
    "test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids": 0.23384582399739884,
    "test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping": 1.3802945329953218,
    "test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case": 0.3421039020031458,
    "test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn": 0.0795805429952452,
    "test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted": 0.16077585499442648,
    "test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure": 0.32440345898794476,
    "test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values": 0.02214915600779932,
    "test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids": 0.00020768599642906338,
    "test_tooling_tests.ModeTests.test_modes_fail_early": 0.01445186800265219,
    "test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping": 0.1651299989898689,
    "test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail": 0.02618941100081429,
    "test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers": 0.0038669939967803657,
    "test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed": 0.021420746998046525,
    "test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd": 0.006824434007285163,
    "test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed": 0.011135132997878827,
    "test_upstream_report.Baseline.test_fork_tree_mismatch_fails": 0.12827010999899358,
    "test_upstream_report.Baseline.test_merge_base_is_never_consulted": 0.12825020198943093,
    "test_upstream_report.Baseline.test_missing_carried_key_fails": 0.08849989199370611,
    "test_upstream_report.Baseline.test_recorded_tree_mismatch_fails": 0.11120857499190606,
    "test_upstream_report.Baseline.test_resolution_needs_no_branches": 0.13615691899030935,
    "test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates": 0.11670836199482437,
    "test_upstream_report.Baseline.test_tag_naming_another_tree_fails": 0.10935756500111893,
    "test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates": 0.15946383299888112,
    "test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail": 0.11195089999819174,
    "test_upstream_report.Classify.test_added_scaffolding_is_class_c": 7.198700041044503e-05,
    "test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b": 3.710101009346545e-05,
    "test_upstream_report.Classify.test_modified_upstream_files_are_class_b": 3.3563992474228144e-05,
    "test_upstream_report.Classify.test_other_additions_are_class_a": 3.3934993552975357e-05,
    "test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one": 0.3193005369976163,
    "test_upstream_report.Cli.test_drift_conflict_exits_one": 0.2816209050070029,
    "test_upstream_report.Cli.test_fresh_report_exits_zero": 0.3252744079945842,
    "test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback": 0.18610384299245197,
    "test_upstream_report.Cli.test_missing_markers_exit_two": 0.22217910499603022,
    "test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits": 0.5145563210098771,
    "test_upstream_report.Cli.test_stale_report_exits_one": 0.19471180699474644,
    "test_upstream_report.Cli.test_truncated_hash_exits_two": 0.17590401199413463,
    "test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one": 0.01858674299728591,
    "test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback": 0.34487051599717233,
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away": 0.08114157899399288,
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one": 0.09177744100452401,
    "test_upstream_report.Conflicts.test_acknowledged_path_passes": 0.01921845698961988,
    "test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result": 0.07581059400399681,
    "test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts": 0.05531146300199907,
    "test_upstream_report.Conflicts.test_clean_status_never_produces_findings": 0.01761506000184454,
    "test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails": 0.017186712997499853,
    "test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths": 0.06664364300377201,
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict": 0.08692347499891184,
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict": 0.085358810989419,
    "test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries": 0.08993123999971431,
    "test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses": 0.019255138002336025,
    "test_upstream_report.Conflicts.test_malformed_informational_section_raises": 0.017851761003839783,
    "test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding": 0.01999963699199725,
    "test_upstream_report.Conflicts.test_unattributed_conflict_fails": 0.019540882000001147,
    "test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path": 0.018489537003915757,
    "test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag": 0.15789049200247973,
    "test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename": 0.14768803700280841,
    "test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit": 0.13047606499458198,
    "test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name": 0.10768183799518738,
    "test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit": 0.16099517200200353,
    "test_upstream_report.Freshness.test_generation_is_a_fixpoint": 0.1494960840063868,
    "test_upstream_report.Freshness.test_regenerated_and_staged_report_passes": 0.14764566501253285,
    "test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose": 0.134811161988182,
    "test_upstream_report.Freshness.test_stale_report_is_a_finding": 0.13889381699846126,
    "test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored": 0.18290891899960116,
    "test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict": 0.14576190400111955,
    "test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error": 0.10436776898859534,
    "test_upstream_report.Inventory.test_inventory_excludes_task_records": 0.1216351759940153,
    "test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files": 0.11586060498666484,
    "test_upstream_report.Inventory.test_inventory_is_sorted_by_path": 0.11966868600575253,
    "test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree": 0.10441547899972647,
    "test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts": 0.10885764000704512,
    "test_upstream_report.Inventory.test_local_block_counts_classes": 0.1365120849950472,
    "test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path": 0.11204201298824046,
    "test_upstream_report.Splice.test_markers_out_of_order_are_an_error": 7.242799620144069e-05,
    "test_upstream_report.Splice.test_missing_markers_are_an_error": 5.765000241808593e-05,
    "test_upstream_report.Splice.test_splice_is_idempotent": 4.3693988118320704e-05,
    "test_upstream_report.Splice.test_splice_replaces_only_between_markers": 3.596900205593556e-05,
    "test_upstream_report.Stage.test_fresh_report_is_left_alone": 0.13822062300459947,
    "test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale": 0.1699350190028781,
    "test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged": 0.15996703099517617,
    "test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes": 0.142860110005131,
    "test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report": 0.14619292500719894,
    "test_vdrag.WireTest.test_fixed_is_24_8": 8.14849918242544e-05,
    "test_vdrag.WireTest.test_header_packs_object_size_and_opcode": 5.826099368277937e-05,
    "test_vdrag.WireTest.test_parse_global": 5.555599636863917e-05,
    "test_vdrag.WireTest.test_split_messages_handles_partial_tail": 6.236899935174733e-05,
    "test_vdrag.WireTest.test_string_is_padded_and_nul_terminated": 4.15290123783052e-05,
    "test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied": 0.0003587950050132349,
    "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded": 8.636998245492578e-06,
    "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt": 4.55799454357475e-06,
    "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json": 4.29799547418952e-06,
    "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched": 4.659013939090073e-06,
    "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores": 4.468995030038059e-06,
    "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails": 3.936991561204195e-06,
    "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions": 4.849003744311631e-06,
    "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt": 4.157991497777402e-06,
    "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails": 3.746987204067409e-06,
    "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed": 3.816996468231082e-06,
    "test_vt_lib.VtLibTests.test_term_while_away_restores_home": 3.827008185908198e-06
  },
  "wall_s": 34.317,
  "mode": "1"
}
```

## amend-ubuntu-check/native.json

```json
{
  "ids": [
    "test_affected.Select.test_docs_and_tasks_select_nothing",
    "test_affected.Select.test_excluded_member_is_never_selected",
    "test_affected.Select.test_member_change_selects_its_dependents",
    "test_affected.Select.test_nothing_changed_selects_nothing",
    "test_affected.Select.test_root_source_selects_the_root_package",
    "test_affected.Select.test_workspace_files_select_everything",
    "test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi",
    "test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry",
    "test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty",
    "test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run",
    "test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run",
    "test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint",
    "test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run",
    "test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process",
    "test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash",
    "test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing",
    "test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source",
    "test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing",
    "test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing",
    "test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash",
    "test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames",
    "test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text",
    "test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses",
    "test_capture_meta.JudgementTests.test_quiet_and_settle",
    "test_capture_meta.JudgementTests.test_threshold_validation",
    "test_capture_meta.LockTests.test_acquire_release_and_ownership_check",
    "test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed",
    "test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder",
    "test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed",
    "test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed",
    "test_capture_meta.LockTests.test_public_read_waits_for_guard",
    "test_capture_meta.LockTests.test_rejects_invalid_owner_fields",
    "test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one",
    "test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients",
    "test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run",
    "test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load",
    "test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients",
    "test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu",
    "test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling",
    "test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock",
    "test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run",
    "test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock",
    "test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight",
    "test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load",
    "test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order",
    "test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list",
    "test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run",
    "test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema",
    "test_capture_meta.RecordTests.test_load_record_rejects_non_object_json",
    "test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation",
    "test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite",
    "test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks",
    "test_capture_meta.SamplingTests.test_sample_stream_and_summary",
    "test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients",
    "test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample",
    "test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse",
    "test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises",
    "test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock",
    "test_capture_meta.SettleTests.test_release_command_is_ownership_checked",
    "test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses",
    "test_capture_meta.SettleTests.test_settled_entry_carries_inputs",
    "test_capture_meta.ShowTests.test_main_show_exit_codes",
    "test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts",
    "test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full",
    "test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full",
    "test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate",
    "test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr",
    "test_gates.Gates.test_focused_recipe_records_failure_through_host_budget",
    "test_gates.Gates.test_focused_recipe_requires_arguments",
    "test_gates.Gates.test_full_recipe_failure_stops_before_following_stages",
    "test_gates.Gates.test_lfs_failure_stops_before_gate",
    "test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast",
    "test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input",
    "test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target",
    "test_gates.Gates.test_staged_paths_select_docs_only_conservatively",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build",
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach",
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam",
    "test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces",
    "test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry",
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light",
    "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end",
    "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused",
    "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound",
    "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release",
    "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops",
    "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch",
    "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases",
    "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart",
    "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch",
    "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item",
    "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size",
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size",
    "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools",
    "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record",
    "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt",
    "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait",
    "test_optic_settling.DriverCleanupTests.test_term_during_export",
    "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus",
    "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release",
    "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client",
    "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools",
    "test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time",
    "test_optic_settling.EdgeTests.test_window_counts_half_open_intervals",
    "test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only",
    "test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup",
    "test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed",
    "test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane",
    "test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime",
    "test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case",
    "test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval",
    "test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request",
    "test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked",
    "test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws",
    "test_optic_settling.RunTests.test_collect_frame_must_redraw",
    "test_optic_settling.RunTests.test_complete_lane_passes",
    "test_optic_settling.RunTests.test_complete_pilot_passes",
    "test_optic_settling.RunTests.test_consumer_frames_and_samples",
    "test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale",
    "test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot",
    "test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus",
    "test_optic_settling.RunTests.test_journal_needs_consistent_alignment",
    "test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases",
    "test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log",
    "test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end",
    "test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls",
    "test_optic_settling.RunTests.test_rejects_heartbeat_gap",
    "test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries",
    "test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases",
    "test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture",
    "test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples",
    "test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace",
    "test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause",
    "test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval",
    "test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window",
    "test_optic_settling.RunTests.test_trace_without_messages",
    "test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return",
    "test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason",
    "test_optic_settling.RunTests.test_zone_names_with_unquoted_commas",
    "test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit",
    "test_package_pin.PinTest.test_check_rejects_a_hand_edited_count",
    "test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline",
    "test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values",
    "test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error",
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error",
    "test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session",
    "test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero",
    "test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged",
    "test_screencast_consumer.PackedRgbTests.test_short_buffer_raises",
    "test_screencast_consumer.PackedRgbTests.test_strips_row_padding",
    "test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl",
    "test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending",
    "test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request",
    "test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending",
    "test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last",
    "test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request",
    "test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption",
    "test_screencast_consumer.WaitForTests.test_times_out_without_a_node",
    "test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared",
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named",
    "test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named",
    "test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes",
    "test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped",
    "test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory",
    "test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared",
    "test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config",
    "test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run",
    "test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail",
    "test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails",
    "test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass",
    "test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution",
    "test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail",
    "test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting",
    "test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery",
    "test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures",
    "test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery",
    "test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids",
    "test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping",
    "test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case",
    "test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn",
    "test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted",
    "test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure",
    "test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values",
    "test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids",
    "test_tooling_tests.ModeTests.test_modes_fail_early",
    "test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping",
    "test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail",
    "test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers",
    "test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed",
    "test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd",
    "test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed",
    "test_upstream_report.Baseline.test_fork_tree_mismatch_fails",
    "test_upstream_report.Baseline.test_merge_base_is_never_consulted",
    "test_upstream_report.Baseline.test_missing_carried_key_fails",
    "test_upstream_report.Baseline.test_recorded_tree_mismatch_fails",
    "test_upstream_report.Baseline.test_resolution_needs_no_branches",
    "test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates",
    "test_upstream_report.Baseline.test_tag_naming_another_tree_fails",
    "test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates",
    "test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail",
    "test_upstream_report.Classify.test_added_scaffolding_is_class_c",
    "test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b",
    "test_upstream_report.Classify.test_modified_upstream_files_are_class_b",
    "test_upstream_report.Classify.test_other_additions_are_class_a",
    "test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one",
    "test_upstream_report.Cli.test_drift_conflict_exits_one",
    "test_upstream_report.Cli.test_fresh_report_exits_zero",
    "test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback",
    "test_upstream_report.Cli.test_missing_markers_exit_two",
    "test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits",
    "test_upstream_report.Cli.test_stale_report_exits_one",
    "test_upstream_report.Cli.test_truncated_hash_exits_two",
    "test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one",
    "test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback",
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away",
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one",
    "test_upstream_report.Conflicts.test_acknowledged_path_passes",
    "test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result",
    "test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts",
    "test_upstream_report.Conflicts.test_clean_status_never_produces_findings",
    "test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails",
    "test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths",
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict",
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict",
    "test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries",
    "test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses",
    "test_upstream_report.Conflicts.test_malformed_informational_section_raises",
    "test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding",
    "test_upstream_report.Conflicts.test_unattributed_conflict_fails",
    "test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path",
    "test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag",
    "test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename",
    "test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit",
    "test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name",
    "test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit",
    "test_upstream_report.Freshness.test_generation_is_a_fixpoint",
    "test_upstream_report.Freshness.test_regenerated_and_staged_report_passes",
    "test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose",
    "test_upstream_report.Freshness.test_stale_report_is_a_finding",
    "test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored",
    "test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict",
    "test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error",
    "test_upstream_report.Inventory.test_inventory_excludes_task_records",
    "test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files",
    "test_upstream_report.Inventory.test_inventory_is_sorted_by_path",
    "test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree",
    "test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts",
    "test_upstream_report.Inventory.test_local_block_counts_classes",
    "test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path",
    "test_upstream_report.Splice.test_markers_out_of_order_are_an_error",
    "test_upstream_report.Splice.test_missing_markers_are_an_error",
    "test_upstream_report.Splice.test_splice_is_idempotent",
    "test_upstream_report.Splice.test_splice_replaces_only_between_markers",
    "test_upstream_report.Stage.test_fresh_report_is_left_alone",
    "test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale",
    "test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged",
    "test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes",
    "test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report",
    "test_vdrag.WireTest.test_fixed_is_24_8",
    "test_vdrag.WireTest.test_header_packs_object_size_and_opcode",
    "test_vdrag.WireTest.test_parse_global",
    "test_vdrag.WireTest.test_split_messages_handles_partial_tail",
    "test_vdrag.WireTest.test_string_is_padded_and_nul_terminated",
    "test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied",
    "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded",
    "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt",
    "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json",
    "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched",
    "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores",
    "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails",
    "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions",
    "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt",
    "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails",
    "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed",
    "test_vt_lib.VtLibTests.test_term_while_away_restores_home"
  ],
  "ran": 320,
  "skipped": [
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code",
      "ImageMagick is required for positive face probes"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate",
      "set MATERIAL_RETAINED_NIRI to validate generated KDL"
    ],
    [
      "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output",
      "ImageMagick is required for pixel probes"
    ],
    [
      "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture",
      "ImageMagick is required for capture decoding"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_during_export",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ],
    [
      "test_vt_lib.VtLibTests.test_term_while_away_restores_home",
      "NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0"
    ]
  ],
  "failures": [],
  "errors": [],
  "expected_failures": [],
  "unexpected_successes": [],
  "elapsed": {
    "test_affected.Select.test_docs_and_tasks_select_nothing": 0.000190281993127428,
    "test_affected.Select.test_excluded_member_is_never_selected": 6.729899905622005e-05,
    "test_affected.Select.test_member_change_selects_its_dependents": 8.834800974000245e-05,
    "test_affected.Select.test_nothing_changed_selects_nothing": 2.3635002435185015e-05,
    "test_affected.Select.test_root_source_selects_the_root_package": 0.0001075750042218715,
    "test_affected.Select.test_workspace_files_select_everything": 6.225900142453611e-05,
    "test_capture_meta.EndToEndTest.test_real_binary_against_fake_nvidia_smi": 2.5737255230051233,
    "test_capture_meta.GpuEvidenceTests.test_clients_and_invalid_telemetry": 0.0005618110008072108,
    "test_capture_meta.GpuEvidenceTests.test_failed_command_reports_stdout_when_stderr_is_empty": 0.0005452699988381937,
    "test_capture_meta.GpuEvidenceTests.test_malformed_gpu_sample_and_cpu_memory_cannot_run": 0.00032982800621539354,
    "test_capture_meta.GpuEvidenceTests.test_missing_or_unsupported_inventory_cannot_run": 0.0003220840007998049,
    "test_capture_meta.HostLoadReaderTests.test_absent_tool_cannot_run_with_install_hint": 0.0005824310064781457,
    "test_capture_meta.HostLoadReaderTests.test_failed_or_malformed_report_cannot_run": 0.007014272996457294,
    "test_capture_meta.HostLoadReaderTests.test_report_runs_the_load_section_and_drops_its_own_process": 0.0026192239893134683,
    "test_capture_meta.IdentityTests.test_dirty_tree_records_diff_hash": 0.03562302100181114,
    "test_capture_meta.IdentityTests.test_git_launch_failure_is_cannot_run_and_writes_nothing": 0.002082671009702608,
    "test_capture_meta.IdentityTests.test_hashes_match_sha256sum_and_records_source": 0.02544998499797657,
    "test_capture_meta.IdentityTests.test_missing_input_cli_refuses_without_writing": 0.0025892260018736124,
    "test_capture_meta.IdentityTests.test_missing_path_refuses_and_writes_nothing": 0.015624100007698871,
    "test_capture_meta.IdentityTests.test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash": 0.037687707008444704,
    "test_capture_meta.IdentityTests.test_untracked_source_handles_unusual_filenames": 0.021464204997755587,
    "test_capture_meta.IdentityTests.test_untracked_symlink_hashes_its_target_text": 0.038791091996245086,
    "test_capture_meta.JudgementTests.test_each_quiet_threshold_and_lane_client_refuses": 0.001331398991169408,
    "test_capture_meta.JudgementTests.test_quiet_and_settle": 0.0005280769983073696,
    "test_capture_meta.JudgementTests.test_threshold_validation": 0.00011829500726889819,
    "test_capture_meta.LockTests.test_acquire_release_and_ownership_check": 0.00151433699647896,
    "test_capture_meta.LockTests.test_binary_corrupt_lock_is_never_reclaimed": 0.001059611007804051,
    "test_capture_meta.LockTests.test_guard_serializes_acquire_against_a_concurrent_holder": 0.30184341200219933,
    "test_capture_meta.LockTests.test_half_written_or_corrupt_lock_is_never_reclaimed": 0.0014108799950918183,
    "test_capture_meta.LockTests.test_held_by_live_pid_refuses_stale_is_reclaimed": 0.001474290998885408,
    "test_capture_meta.LockTests.test_public_read_waits_for_guard": 0.3014420160034206,
    "test_capture_meta.LockTests.test_rejects_invalid_owner_fields": 0.0022013470006641,
    "test_capture_meta.LockTests.test_two_processes_racing_admit_exactly_one": 0.07527582500188146,
    "test_capture_meta.PreflightTests.test_busy_machine_refuses_and_records_reasons_and_clients": 0.005105855001602322,
    "test_capture_meta.PreflightTests.test_cli_validation_returns_two_without_mutating_run": 0.002153856010409072,
    "test_capture_meta.PreflightTests.test_cpu_or_load_refusal_records_and_names_the_load": 0.009678093003458343,
    "test_capture_meta.PreflightTests.test_dedicated_requires_tty_and_no_clients": 0.007623104000231251,
    "test_capture_meta.PreflightTests.test_gpu_only_refusal_does_not_ask_who_loads_the_cpu": 0.0026715529966168106,
    "test_capture_meta.PreflightTests.test_held_lock_refuses_without_sampling": 0.001309336003032513,
    "test_capture_meta.PreflightTests.test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock": 0.0024574249982833862,
    "test_capture_meta.PreflightTests.test_invalid_seconds_or_pid_does_not_mutate_run": 0.0009392200008733198,
    "test_capture_meta.PreflightTests.test_missing_tool_cannot_run_and_releases_lock": 0.0021339480008464307,
    "test_capture_meta.PreflightTests.test_quiet_headless_writes_run_environment_baseline_preflight": 0.0027397230005590245,
    "test_capture_meta.PreflightTests.test_quiet_preflight_does_not_run_host_load": 0.0025948979891836643,
    "test_capture_meta.RecordTests.test_append_sub_run_accumulates_in_order": 0.0005757379985880107,
    "test_capture_meta.RecordTests.test_append_sub_run_rejects_non_list": 0.0003817779943346977,
    "test_capture_meta.RecordTests.test_expected_read_failures_are_cannot_run": 0.0002914060023613274,
    "test_capture_meta.RecordTests.test_load_record_rejects_malformed_and_unknown_schema": 0.00048782999510876834,
    "test_capture_meta.RecordTests.test_load_record_rejects_non_object_json": 0.00030892899667378515,
    "test_capture_meta.RecordTests.test_malformed_sections_make_cli_consumers_exit_two_without_mutation": 0.30784075599513017,
    "test_capture_meta.RecordTests.test_write_section_creates_record_and_refuses_rewrite": 0.0009962499898392707,
    "test_capture_meta.SamplingTests.test_proc_reader_does_not_double_count_guest_ticks": 0.0012899689900223166,
    "test_capture_meta.SamplingTests.test_sample_stream_and_summary": 0.00016733899246901274,
    "test_capture_meta.SamplingTests.test_summary_medians_iqr_and_clients": 0.0001337349967798218,
    "test_capture_meta.SettleTests.test_invalid_seconds_does_not_mutate_or_sample": 0.005021283999667503,
    "test_capture_meta.SettleTests.test_missing_input_and_foreign_lock_refuse": 0.005645474011544138,
    "test_capture_meta.SettleTests.test_off_baseline_appends_refused_and_raises": 0.005828442997881211,
    "test_capture_meta.SettleTests.test_refused_before_acquire_does_not_release_foreign_lock": 0.0035455699980957434,
    "test_capture_meta.SettleTests.test_release_command_is_ownership_checked": 0.010887789001571946,
    "test_capture_meta.SettleTests.test_same_run_id_with_wrong_owner_refuses": 0.0061752039910061285,
    "test_capture_meta.SettleTests.test_settled_entry_carries_inputs": 0.0058025939943036065,
    "test_capture_meta.ShowTests.test_main_show_exit_codes": 0.004689131004852243,
    "test_capture_meta.ShowTests.test_render_lists_environment_provenance_and_verdicts": 8.878899097908288e-05,
    "test_gates.Gates.test_commit_classification_errors_and_empty_index_take_full": 0.15460649100714363,
    "test_gates.Gates.test_deleted_subject_and_both_rename_endpoints_take_full": 0.1885810989915626,
    "test_gates.Gates.test_failing_remote_evaluation_with_partial_output_takes_full_gate": 0.018165715999202803,
    "test_gates.Gates.test_focused_recipe_preserves_arguments_and_counts_stderr": 0.10500834499543998,
    "test_gates.Gates.test_focused_recipe_records_failure_through_host_budget": 0.10373803300899453,
    "test_gates.Gates.test_focused_recipe_requires_arguments": 0.011926860010134988,
    "test_gates.Gates.test_full_recipe_failure_stops_before_following_stages": 0.0948689920041943,
    "test_gates.Gates.test_lfs_failure_stops_before_gate": 0.0487492679967545,
    "test_gates.Gates.test_narrow_subjects_select_full_and_unrelated_paths_select_fast": 1.250331417992129,
    "test_gates.Gates.test_push_selects_ci_gate_and_preserves_lfs_input": 0.23625997599447146,
    "test_gates.Gates.test_recipe_composition_modes_counts_and_shared_hook_target": 1.0086451649985975,
    "test_gates.Gates.test_staged_paths_select_docs_only_conservatively": 0.856857566992403,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_cleanup_ignores_capture_release_refusal": 0.00321628300298471,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_finish_hashes_every_file_but_the_manifest": 0.01124115299899131,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_fails_when_the_gpu_never_idles": 0.6645883369928924,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_gpu_cooldown_waits_for_six_consecutive_p8_polls": 0.050248176004970446,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_preflight_and_identity_pass_complete_capture_arguments": 0.004421470002853312,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_passes_config_and_name": 0.006279392997385003,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_settle_before_launch_skips_the_cooldown_under_a_capture_meta_stub": 0.005550443005631678,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_smokes_preflight_after_sourcing_and_identify_after_build": 0.0004262729926267639,
    "test_glass_optic_smoke.CaptureMetaAdoptionTest.test_start_nested_settles_first_and_lib_never_preflights": 0.0005344490054994822,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_rejects_shader_fallback_before_accepting_metrics": 0.0002218529989477247,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_additive_uses_face_for_aurora_and_chamfer_for_surface_lights": 0.01318253499630373,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_serializes_default_opacity_as_kdl_float": 0.00434661700273864,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_capture_validates_the_generated_single_response_config_before_launch": 0.012357861996861175,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_cost_scope_rejects_capture_override_before_preflight": 0.003579914991860278,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_explicit_retained_candidate_inputs_skip_build": 0.0021370549948187545,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_propagates_strict_settle_refusal_after_wait": 0.00589012099953834,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_gpu_trace_waits_before_strict_settle_then_checks_log_and_median": 0.007528354006353766,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_matrix_pins_required_fixture_values": 0.00023390499700326473,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_aurora_gpu_case_cools_before_trace": 0.006150857007014565,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_motion_moves_the_tracked_rightmost_column": 0.00044653100485447794,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_records_per_frame_motion_reach_timing_and_geometry": 0.0004981999954907224,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate": 1.0449992259964347e-05,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_rest_measurement_pins_roughness_and_response_values": 0.00014257100701797754,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_writes_all_baseline_rest_selector_and_motion_configs": 0.2413614709948888,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code": 1.7974001821130514e-05,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate": 3.918001311831176e-06,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_resize_flex_uses_frozen_clock_check_without_capture_setup": 0.014358155996887945,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_scope_dispatch_never_runs_cost_from_pixel_scope": 0.005867277010111138,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output": 9.148003300651908e-06,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_verify_runs_the_focused_pixel_matrix_in_order": 0.002402251004241407,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_generates_one_distortion_node_and_attenuation_skips_reach_bound": 0.07302688701020088,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_aurora_motion_records_frames_and_ipc_geometry_without_repeat_gate": 0.00046722000115551054,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_matrix_prepares_every_offline_fixture": 0.0021365129941841587,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_measurements_pass_the_profile_and_interval_inputs_to_reach": 0.01490016900061164,
    "test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_within_ring_reuses_the_default_pinned_beam": 0.0006435170071199536,
    "test_glass_optic_smoke.TraceCoverageTest.test_complete_idle_and_four_hz_traces": 0.030315081006847322,
    "test_glass_optic_smoke.TraceCoverageTest.test_empty_truncated_and_stalled_traces_fail": 0.0399545090040192,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture": 9.216993930749595e-06,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_profile_and_attenuation_use_decoded_quantization_intervals": 0.014257955001085065,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_is_measured_from_the_face_edge": 0.344763884000713,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta": 0.0007028100080788136,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_reach_uses_scatter_bound_and_rounded_slab_geometry": 0.3065387609967729,
    "test_glass_render_order_metrics.RenderOrderMetricsTest.test_signed_grain_and_quantized_additive_light": 0.02211964500020258,
    "test_optic_settling.DriverCleanupTests.test_a_drm_niri_that_ignores_term_is_killed_at_case_end": 4.397996235638857e-06,
    "test_optic_settling.DriverCleanupTests.test_a_leftover_niri_from_an_earlier_run_is_refused": 2.5040062610059977e-06,
    "test_optic_settling.DriverCleanupTests.test_a_lock_client_that_ignores_unlock_fails_within_the_bound": 2.505010343156755e-06,
    "test_optic_settling.DriverCleanupTests.test_a_refused_return_fails_the_run_after_release": 2.22500239033252e-06,
    "test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops": 1.9029976101592183e-06,
    "test_optic_settling.DriverCleanupTests.test_a_second_connected_output_is_refused_before_launch": 2.173997927457094e-06,
    "test_optic_settling.DriverCleanupTests.test_a_second_term_during_cleanup_still_restores_and_releases": 2.0839943317696452e-06,
    "test_optic_settling.DriverCleanupTests.test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart": 9.088005754165351e-06,
    "test_optic_settling.DriverCleanupTests.test_an_unconnected_drm_output_is_refused_before_launch": 1.714011887088418e-06,
    "test_optic_settling.DriverCleanupTests.test_dedicated_prerequisites_name_the_missing_item": 1.9129947759211063e-06,
    "test_optic_settling.DriverCleanupTests.test_failed_preflight_releases_and_keeps_its_record": 2.054002834483981e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_dead_consumer": 1.9139988580718637e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_small_probe": 1.7529964679852128e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_probe_size": 1.6429985407739878e-06,
    "test_optic_settling.DriverCleanupTests.test_screencast_refuses_wrong_sample_size": 1.7529964679852128e-06,
    "test_optic_settling.DriverCleanupTests.test_stub_overrides_need_stub_tools": 2.14400643017143e-06,
    "test_optic_settling.DriverCleanupTests.test_stub_tools_need_a_stub_capture_record": 1.7530110199004412e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt": 1.8739956431090832e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_capture_stops_the_schedule_wait": 1.6130070434883237e-06,
    "test_optic_settling.DriverCleanupTests.test_term_during_export": 1.9129947759211063e-06,
    "test_optic_settling.DriverCleanupTests.test_term_while_casting_reaps_the_consumer_and_the_bus": 2.1140003809705377e-06,
    "test_optic_settling.DriverCleanupTests.test_term_while_switched_away_restores_the_vt_before_release": 1.9230064935982227e-06,
    "test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client": 1.7339916666969657e-06,
    "test_optic_settling.DriverCleanupTests.test_vt_overrides_need_stub_tools": 1.824009814299643e-06,
    "test_optic_settling.EdgeTests.test_pause_resume_preserve_logical_time": 0.00013828299415763468,
    "test_optic_settling.EdgeTests.test_window_counts_half_open_intervals": 5.393200262915343e-05,
    "test_optic_settling.IdentityTests.test_run_identity_ignores_the_run_directory_only": 0.0020350300037534907,
    "test_optic_settling.JournalSignalTests.test_term_during_journal_end_preserves_exit_and_cleanup": 0.005225182991125621,
    "test_optic_settling.LifecycleCoverageTests.test_subject_paths_are_full_routed": 0.16944356900057755,
    "test_optic_settling.PrepareTests.test_prepare_the_dedicated_lane": 0.2506098949961597,
    "test_optic_settling.PrepareTests.test_prepare_validates_inventory_and_cleans_runtime": 0.40295076499751303,
    "test_optic_settling.RunTests.test_a_malformed_cast_frame_names_its_case": 0.010299397996277548,
    "test_optic_settling.RunTests.test_a_redraw_between_samples_breaks_the_quiet_interval": 0.004940420010825619,
    "test_optic_settling.RunTests.test_a_sample_must_be_the_first_cast_frame_after_its_request": 0.010727333996328525,
    "test_optic_settling.RunTests.test_an_empty_consumer_is_still_checked": 0.015383090008981526,
    "test_optic_settling.RunTests.test_bounds_the_edge_flush_and_settled_redraws": 0.007046305006952025,
    "test_optic_settling.RunTests.test_collect_frame_must_redraw": 0.00330317800398916,
    "test_optic_settling.RunTests.test_complete_lane_passes": 0.013312782000866719,
    "test_optic_settling.RunTests.test_complete_pilot_passes": 0.002878818995668553,
    "test_optic_settling.RunTests.test_consumer_frames_and_samples": 0.006151377994683571,
    "test_optic_settling.RunTests.test_dedicated_topology_pins_output_mode_and_scale": 0.01509387799887918,
    "test_optic_settling.RunTests.test_development_subset_never_passes_as_a_pilot": 0.0046999609912745655,
    "test_optic_settling.RunTests.test_edge_must_fall_inside_its_named_stimulus": 0.005372493003960699,
    "test_optic_settling.RunTests.test_journal_needs_consistent_alignment": 0.0026737769949249923,
    "test_optic_settling.RunTests.test_lane_verdict_exits_zero_and_lists_unverified_cases": 0.00677293399348855,
    "test_optic_settling.RunTests.test_rejects_a_panic_in_the_compositor_log": 0.004118352007935755,
    "test_optic_settling.RunTests.test_rejects_a_trace_that_stops_before_the_declared_end": 0.0066163949959445745,
    "test_optic_settling.RunTests.test_rejects_absent_edges_headers_export_and_controls": 0.008879150002030656,
    "test_optic_settling.RunTests.test_rejects_heartbeat_gap": 0.0029584199946839362,
    "test_optic_settling.RunTests.test_rejects_malformed_edge_in_entries": 0.010989714006427675,
    "test_optic_settling.RunTests.test_rejects_missing_duplicate_short_and_false_hardware_cases": 0.013557879996369593,
    "test_optic_settling.RunTests.test_rejects_short_trace_against_declared_capture": 0.005227696994552389,
    "test_optic_settling.RunTests.test_rejects_stale_missing_and_late_samples": 0.006931345007615164,
    "test_optic_settling.RunTests.test_rejects_stimulus_after_the_trace": 0.0026994159998139367,
    "test_optic_settling.RunTests.test_setup_segment_is_unchecked_only_before_a_pause": 0.004792648003785871,
    "test_optic_settling.RunTests.test_stimulus_exempts_only_its_interval": 0.002837780994013883,
    "test_optic_settling.RunTests.test_stimulus_requires_its_messages_inside_its_window": 0.004998069009161554,
    "test_optic_settling.RunTests.test_trace_without_messages": 0.004074969998328015,
    "test_optic_settling.RunTests.test_tty_resume_needs_the_resume_edge_inside_vt_return": 0.0056731270015006885,
    "test_optic_settling.RunTests.test_unverified_lanes_carry_their_reason": 0.002959642995847389,
    "test_optic_settling.RunTests.test_zone_names_with_unquoted_commas": 0.002951747999759391,
    "test_package_pin.PinTest.test_check_accepts_a_pkgbuild_that_describes_its_own_commit": 0.11945427699538413,
    "test_package_pin.PinTest.test_check_rejects_a_hand_edited_count": 0.1141185430024052,
    "test_package_pin.PinTest.test_the_count_is_the_commits_this_fork_carries_over_the_baseline": 0.06500383900129236,
    "test_package_pin.PinTest.test_writing_a_pin_replaces_all_three_values": 0.06785875200876035,
    "test_screencast_consumer.ConsumerEndToEndTests.test_startup_sampling_and_summary": 0.33614744400256313,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero": 0.22424986500118393,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error": 0.22709020000183955,
    "test_screencast_consumer.ConsumerFailureTests.test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error": 0.3024583699880168,
    "test_screencast_consumer.ConsumerFailureTests.test_term_while_waiting_for_the_node_still_stops_the_session": 0.27572021100786515,
    "test_screencast_consumer.ConsumerFailureTests.test_unreadable_request_file_exits_nonzero": 0.23455162599566393,
    "test_screencast_consumer.PackedRgbTests.test_packed_input_is_unchanged": 6.157701136544347e-05,
    "test_screencast_consumer.PackedRgbTests.test_short_buffer_raises": 8.112500654533505e-05,
    "test_screencast_consumer.PackedRgbTests.test_strips_row_padding": 4.3352003558538854e-05,
    "test_screencast_consumer.PipelineTests.test_imports_dma_bufs_through_gl": 4.2530999053269625e-05,
    "test_screencast_consumer.SamplerTests.test_a_failed_armed_write_leaves_nothing_pending": 0.00046864298928994685,
    "test_screencast_consumer.SamplerTests.test_nothing_is_saved_without_a_request": 0.0003040999872609973,
    "test_screencast_consumer.SamplerTests.test_refuses_a_second_request_while_one_is_pending": 0.0005153020028956234,
    "test_screencast_consumer.SamplerTests.test_samples_are_written_atomically_raw_first_json_last": 0.0017942309932550415,
    "test_screencast_consumer.SamplerTests.test_saves_the_first_frame_after_the_request": 0.0011608930071815848,
    "test_screencast_consumer.WaitForTests.test_the_discovery_deadline_cannot_stop_consumption": 0.7513737419940298,
    "test_screencast_consumer.WaitForTests.test_times_out_without_a_node": 0.20070805899740662,
    "test_target_dir_check.TargetDirCheckTest.test_a_shared_build_dir_is_shared": 0.1374973689962644,
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_whose_target_cannot_be_read_is_skipped": 0.1956556250079302,
    "test_target_dir_check.TargetDirCheckTest.test_another_checkout_with_a_broken_manifest_is_skipped": 0.14213566199759953,
    "test_target_dir_check.TargetDirCheckTest.test_cargo_build_target_dir_in_the_environment_is_named": 0.13320449300226755,
    "test_target_dir_check.TargetDirCheckTest.test_cargo_target_dir_in_the_environment_is_named": 0.13948711199918762,
    "test_target_dir_check.TargetDirCheckTest.test_locked_worktree_whose_directory_is_gone_is_skipped": 0.19717790800496005,
    "test_target_dir_check.TargetDirCheckTest.test_main_checkout_warns_but_passes": 0.1223134380124975,
    "test_target_dir_check.TargetDirCheckTest.test_missing_worktree_is_skipped": 0.19853831100044772,
    "test_target_dir_check.TargetDirCheckTest.test_runs_from_a_subdirectory": 0.235300363987335,
    "test_target_dir_check.TargetDirCheckTest.test_same_dir_through_a_symlink_is_shared": 0.16247042300528847,
    "test_target_dir_check.TargetDirCheckTest.test_the_hint_overrides_a_shared_parent_config": 0.2938536430010572,
    "test_target_dir_check.TargetDirCheckTest.test_this_checkout_with_a_broken_manifest_cannot_run": 0.10285517800366506,
    "test_target_dir_check.TargetDirCheckTest.test_two_worktrees_sharing_a_third_dir_both_fail": 0.29583578299207147,
    "test_target_dir_check.TargetDirCheckTest.test_worktree_borrowing_the_main_target_fails": 0.2405043950129766,
    "test_target_dir_check.TargetDirCheckTest.test_worktrees_with_their_own_target_pass": 0.3040684500010684,
    "test_tooling_tests.CoordinatorTests.test_ci_rejects_required_class_skip_before_execution": 0.06273049500305206,
    "test_tooling_tests.CoordinatorTests.test_dynamic_skip_failure_import_error_and_crash_fail": 0.47597732799476944,
    "test_tooling_tests.CoordinatorTests.test_expected_failure_preserves_native_reporting": 0.13962592600728385,
    "test_tooling_tests.CoordinatorTests.test_fast_malformed_budget_fails_before_discovery": 0.06792362598935142,
    "test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures": 0.30797988700214773,
    "test_tooling_tests.CoordinatorTests.test_fast_rejects_ci_and_public_worker_mode_before_discovery": 0.13256874000944663,
    "test_tooling_tests.CoordinatorTests.test_full_overrides_fast_and_budget_one_preserves_all_ids": 0.17995551000058185,
    "test_tooling_tests.CoordinatorTests.test_interrupt_during_normal_cleanup_finishes_reaping": 1.3123235759994714,
    "test_tooling_tests.CoordinatorTests.test_interrupt_reaps_a_child_from_an_unwound_case": 0.32208775500475895,
    "test_tooling_tests.CoordinatorTests.test_malformed_budget_fails_before_discovery_or_spawn": 0.06682257598731667,
    "test_tooling_tests.CoordinatorTests.test_optional_skip_is_permitted": 0.13183949000085704,
    "test_tooling_tests.CoordinatorTests.test_unexpected_success_preserves_native_failure": 0.272880072996486,
    "test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values": 0.0021410320041468367,
    "test_tooling_tests.ModeTests.test_flatten_rejects_duplicate_native_ids": 0.00016620699898339808,
    "test_tooling_tests.ModeTests.test_modes_fail_early": 0.0010160370002267882,
    "test_tooling_tests.ModeTests.test_raw_imports_validate_before_skipping": 0.13902937900274992,
    "test_tooling_tests.StaticCoverageTests.test_missing_source_cycle_dynamic_operand_and_missing_module_fail": 0.03314453399798367,
    "test_tooling_tests.StaticCoverageTests.test_nested_command_substitution_in_arithmetic_is_rejected_in_helpers": 0.004629988005035557,
    "test_tooling_tests.StaticCoverageTests.test_package_initializers_and_their_imports_must_be_routed": 0.024369524006033316,
    "test_tooling_tests.StaticCoverageTests.test_sourced_helper_inherits_driver_here_and_repository_cwd": 0.007274318995769136,
    "test_tooling_tests.StaticCoverageTests.test_sources_class_paths_and_transitive_imports_must_be_routed": 0.0171564010088332,
    "test_upstream_report.Baseline.test_fork_tree_mismatch_fails": 0.09791575999406632,
    "test_upstream_report.Baseline.test_merge_base_is_never_consulted": 0.09837694899761118,
    "test_upstream_report.Baseline.test_missing_carried_key_fails": 0.06859298200288322,
    "test_upstream_report.Baseline.test_recorded_tree_mismatch_fails": 0.08370931301033124,
    "test_upstream_report.Baseline.test_resolution_needs_no_branches": 0.10604182300448883,
    "test_upstream_report.Baseline.test_rewritten_ancestry_with_identical_tree_validates": 0.0943137619906338,
    "test_upstream_report.Baseline.test_tag_naming_another_tree_fails": 0.08451717300340533,
    "test_upstream_report.Baseline.test_tag_resolving_to_another_commit_with_the_same_tree_validates": 0.11883708900131751,
    "test_upstream_report.Baseline.test_wrong_carried_patch_ids_fail": 0.09378186801041011,
    "test_upstream_report.Classify.test_added_scaffolding_is_class_c": 8.635500853415579e-05,
    "test_upstream_report.Classify.test_deleted_and_renamed_upstream_files_are_class_b": 4.031599382869899e-05,
    "test_upstream_report.Classify.test_modified_upstream_files_are_class_b": 3.546800871845335e-05,
    "test_upstream_report.Classify.test_other_additions_are_class_a": 3.5528006264939904e-05,
    "test_upstream_report.Cli.test_bad_upstream_ref_exits_two_not_one": 0.24263973599590827,
    "test_upstream_report.Cli.test_drift_conflict_exits_one": 0.22098747199925128,
    "test_upstream_report.Cli.test_fresh_report_exits_zero": 0.23757946699333843,
    "test_upstream_report.Cli.test_malformed_toml_exits_two_without_a_traceback": 0.14841083600185812,
    "test_upstream_report.Cli.test_missing_markers_exit_two": 0.17420305099221878,
    "test_upstream_report.Cli.test_stage_makes_the_check_pass_and_refuses_unstaged_edits": 0.37178187200333923,
    "test_upstream_report.Cli.test_stale_report_exits_one": 0.1605320259986911,
    "test_upstream_report.Cli.test_truncated_hash_exits_two": 0.13216172200918663,
    "test_upstream_report.Cli.test_unexpected_exception_exits_two_not_one": 0.018964397997478954,
    "test_upstream_report.Cli.test_wrong_typed_config_exits_two_without_a_traceback": 0.2711943079921184,
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_away": 0.0796165390056558,
    "test_upstream_report.Conflicts.test_a_directory_cannot_be_acknowledged_beside_a_real_one": 0.06753964300150983,
    "test_upstream_report.Conflicts.test_acknowledged_path_passes": 0.01446989900432527,
    "test_upstream_report.Conflicts.test_bad_ref_is_an_error_not_a_clean_result": 0.05499103700276464,
    "test_upstream_report.Conflicts.test_clean_merge_reports_no_conflicts": 0.0364265219977824,
    "test_upstream_report.Conflicts.test_clean_status_never_produces_findings": 0.013496553001459688,
    "test_upstream_report.Conflicts.test_conflict_notice_on_an_unlisted_path_fails": 0.014072410995140672,
    "test_upstream_report.Conflicts.test_conflict_reports_exit_one_and_the_exact_paths": 0.05901659300434403,
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_beside_an_acknowledged_conflict": 0.06729439500486478,
    "test_upstream_report.Conflicts.test_directory_rename_split_fails_the_verdict": 0.06948417999956291,
    "test_upstream_report.Conflicts.test_directory_rename_split_has_no_conflicted_file_entries": 0.06517820099543314,
    "test_upstream_report.Conflicts.test_empty_result_tree_is_an_error_for_both_protocol_statuses": 0.016144951994647272,
    "test_upstream_report.Conflicts.test_malformed_informational_section_raises": 0.011894919007318094,
    "test_upstream_report.Conflicts.test_unacknowledged_path_is_a_finding": 0.014821149001363665,
    "test_upstream_report.Conflicts.test_unattributed_conflict_fails": 0.012359364991425537,
    "test_upstream_report.Conflicts.test_unattributed_conflict_is_not_hidden_by_an_acknowledged_path": 0.014872355997795239,
    "test_upstream_report.Drift.test_churn_counts_from_the_fork_baseline_commit_not_the_tag": 0.12911529499979224,
    "test_upstream_report.Drift.test_churn_follows_the_baseline_path_across_a_fork_rename": 0.11432858199987095,
    "test_upstream_report.Drift.test_resolve_baseline_exposes_the_fork_baseline_commit": 0.10788912500720471,
    "test_upstream_report.Drift.test_seam_paths_map_to_the_baseline_name": 0.0764158569945721,
    "test_upstream_report.Freshness.test_fresh_checkout_with_empty_staging_area_validates_the_commit": 0.1268306889978703,
    "test_upstream_report.Freshness.test_generation_is_a_fixpoint": 0.11918346899619792,
    "test_upstream_report.Freshness.test_regenerated_and_staged_report_passes": 0.12284777400782332,
    "test_upstream_report.Freshness.test_regeneration_preserves_unstaged_prose": 0.10857636599394027,
    "test_upstream_report.Freshness.test_stale_report_is_a_finding": 0.11270121899724472,
    "test_upstream_report.Freshness.test_unstaged_baseline_edit_is_ignored": 0.14029605299583636,
    "test_upstream_report.Freshness.test_unstaged_source_change_does_not_affect_the_verdict": 0.11928683599398937,
    "test_upstream_report.Inventory.test_disagreeing_diff_formats_are_an_error": 0.08136930099863093,
    "test_upstream_report.Inventory.test_inventory_excludes_task_records": 0.09246756100037601,
    "test_upstream_report.Inventory.test_inventory_excludes_the_tools_own_files": 0.09250127500854433,
    "test_upstream_report.Inventory.test_inventory_is_sorted_by_path": 0.09018052100145724,
    "test_upstream_report.Inventory.test_inventory_reads_the_index_not_the_working_tree": 0.09652582999842707,
    "test_upstream_report.Inventory.test_inventory_reports_status_path_and_line_counts": 0.09132012299960479,
    "test_upstream_report.Inventory.test_local_block_counts_classes": 0.1005256950011244,
    "test_upstream_report.Inventory.test_rename_joins_both_diff_formats_on_the_new_path": 0.09433413999795448,
    "test_upstream_report.Splice.test_markers_out_of_order_are_an_error": 7.75279913796112e-05,
    "test_upstream_report.Splice.test_missing_markers_are_an_error": 4.6397995902225375e-05,
    "test_upstream_report.Splice.test_splice_is_idempotent": 4.8371992306783795e-05,
    "test_upstream_report.Splice.test_splice_replaces_only_between_markers": 3.98960110032931e-05,
    "test_upstream_report.Stage.test_fresh_report_is_left_alone": 0.12259317000280134,
    "test_upstream_report.Stage.test_pathspec_commit_index_is_refused_only_when_stale": 0.12965529499342665,
    "test_upstream_report.Stage.test_stale_report_is_regenerated_and_staged": 0.11372512200614437,
    "test_upstream_report.Stage.test_unstaged_report_edit_is_refused_and_nothing_changes": 0.10060458500811365,
    "test_upstream_report.Stage.test_unstaged_source_change_stays_out_of_the_staged_report": 0.11814307500026189,
    "test_vdrag.WireTest.test_fixed_is_24_8": 7.825899228919297e-05,
    "test_vdrag.WireTest.test_header_packs_object_size_and_opcode": 5.418399814516306e-05,
    "test_vdrag.WireTest.test_parse_global": 5.153799429535866e-05,
    "test_vdrag.WireTest.test_split_messages_handles_partial_tail": 6.53040042379871e-05,
    "test_vdrag.WireTest.test_string_is_padded_and_nul_terminated": 4.0297003579325974e-05,
    "test_vdrag.WireTest.test_stripes_fill_the_buffer_translucent_and_premultiplied": 0.00030433999199885875,
    "test_vt_lib.VtLibTests.test_a_hanging_chvt_is_bounded": 6.551999831572175e-06,
    "test_vt_lib.VtLibTests.test_failed_restore_reports_the_observed_vt": 4.168003215454519e-06,
    "test_vt_lib.VtLibTests.test_failed_restore_with_an_unreadable_active_vt_is_valid_json": 3.636989276856184e-06,
    "test_vt_lib.VtLibTests.test_restore_not_needed_when_never_switched": 3.4460099413990974e-06,
    "test_vt_lib.VtLibTests.test_restore_with_an_unreadable_active_vt_still_restores": 3.3569958759471774e-06,
    "test_vt_lib.VtLibTests.test_spare_fails_loudly_when_loginctl_fails": 3.1460076570510864e-06,
    "test_vt_lib.VtLibTests.test_spare_skips_home_and_logind_sessions": 3.496999852359295e-06,
    "test_vt_lib.VtLibTests.test_spare_skips_the_home_vt": 3.435998223721981e-06,
    "test_vt_lib.VtLibTests.test_spare_without_a_free_vt_fails": 3.1959934858605266e-06,
    "test_vt_lib.VtLibTests.test_switch_verifies_that_the_vt_landed": 3.3159885788336396e-06,
    "test_vt_lib.VtLibTests.test_term_while_away_restores_home": 4.11800283472985e-06
  },
  "wall_s": 26.57,
  "mode": "1"
}
```
