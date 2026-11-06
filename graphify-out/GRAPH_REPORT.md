# Graph Report - m91-backend  (2026-11-06)

## Corpus Check
- 451 files · ~638,421 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10925 nodes · 30390 edges · 412 communities (370 shown, 7 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 752 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `e366e116`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Result
- SqliteStore
- Self
- mex_operations.rs
- message/tests.rs
- SignalStore
- SenderKeyName
- rust-backend/src/send.rs
- iq/groups.rs
- AliceSignalProtocolParameters
- aes_gcm.rs
- ProtocolAddress
- MockBackend
- protocol.rs
- capturing_client
- Event
- AttrParser<'a>
- text_msg
- iq/usync.rs
- send_receive_benchmark.rs
- send/tests.rs
- HashState
- CryptoProviderError
- Error
- src/receipt.rs
- rust-backend AGENTS guide
- message_processing.rs
- stanza/call.rs
- crypto/hash.rs
- ServiceId
- client/tests.rs
- PreKeyId
- iq/tctoken.rs
- iq/prekeys.rs
- Vec
- session_cipher.rs
- wacore/src/upload.rs
- messages.rs
- src/node.rs
- blocklist.rs
- wacore/src/download.rs
- client/lid_pn.rs
- features/message_edit.rs
- create_test_client
- wacore/src/history_sync.rs
- rust-backend/src/download.rs
- proto_helpers.rs
- stanza/groups.rs
- sticker_pack.rs
- Runtime
- NodeStr
- expand_app_state_keys
- send_and_expect_text
- binary_benchmark.rs
- GenericSignedPreKey
- PreKeyBundle
- MexError
- Device
- privacy.rs
- SenderKeyState
- SignalStoreCache
- rust-backend/src/handshake.rs
- encrypt_bot_message
- rust-backend/src/prekeys.rs
- src/retry.rs
- Client
- ratchet/keys.rs
- props.rs
- SessionRecord
- Client
- notification/mod.rs
- signal_adapter.rs
- state.rs
- StanzaHandler
- .new
- Arc
- framing.rs
- GroupInfo
- Store
- PortableCache<K, V>
- store/device.rs
- iq/contacts.rs
- features/presence.rs
- handshake_integration.rs
- .fmt
- wacore/src/pair_code.rs
- MockSendContextResolver
- String
- bridge.rs
- BinaryError
- OwnedNodeRef
- NodeContent
- session_divergence.rs
- stanza/devices.rs
- .new
- src/message.rs
- .new
- pdo.rs
- Jid
- polls.rs
- LidPnCache
- SessionManager
- companion_reg.rs
- prepare_dm_retry_stanza
- TestClient
- src/msg_secret.rs
- Controller
- MsgSecretWriteBuffer
- sessions.rs
- .new
- encrypt.rs
- Jid
- LazyHistorySync
- rust-backend/src/history_sync.rs
- Client
- Client
- Client
- .drop
- .handle_receipt
- BenchResults
- decoder.rs
- encoder.rs
- Controller
- adv.rs
- wacore/src/pair.rs
- src/time.rs
- types/message.rs
- rust-backend/src/upload.rs
- features/community.rs
- types/spam_report.rs
- tests/media.rs
- Encoder<'a, W>
- frame.rs
- Controller
- derive/src/lib.rs
- zlib_pool.rs
- Api
- iq/mediaconn.rs
- types/jid.rs
- views.rs
- AbPropsCache
- InvalidSessionError
- UnifiedSessionManager
- TokioWebSocketTransportFactory
- HttpClient
- iq/chatstate.rs
- GroupMetadata
- portable_cache.rs
- discrepancy_pocs.rs
- src/jid.rs
- features/mex.rs
- Filters
- stanza/business.rs
- dirty.rs
- Jid
- protocol/retry.rs
- .write_string
- StoreError
- prepare_group_stanza
- xml.rs
- SessionState
- MessageContext
- DeviceListRecord
- wacore/src/usync.rs
- wacore/src/prekeys.rs
- UreqHttpClient
- MessageInternalFields
- MessageInfo
- WAPatchName
- features/status.rs
- FlushScope
- Option
- PrivateKey
- features/contacts.rs
- Client
- Client
- ui/tests.rs
- UploadResponse
- resend_rate_limiter.rs
- wacore/src/store/commands.rs
- CapturingMockTransport
- PrivacyCategory
- reporting_token.rs
- stanza/receipt.rs
- PublicKey
- notification/device.rs
- Node
- PersistenceManager
- features/chat_actions.rs
- Inner
- AudioPlayer
- offline_resume.rs
- JidExt
- setup_session
- features/events.rs
- CacheConfig
- memory_soak.rs
- labels.rs
- MediaEncryptor
- Client
- NodeRef
- NoiseSocket
- PatchList
- poll.rs
- util/jid.rs
- rust-backend/src/lib.rs
- src/keepalive.rs
- Option
- src/hash.rs
- src/ib.rs
- NodeRef
- src/message_edit.rs
- secret_enc_addon.rs
- PairCodeUtils
- generate_reporting_token_content
- Persist
- BotBuilder<B, T, H, R>
- IqError
- ClientProfile
- Client
- aes_256_cbc_encrypt_into
- controller.rs
- RetryReason
- telemetry.rs
- ClientError
- reporting_token_benchmark.rs
- curve.rs
- Message
- wire_enum_serde_test.rs
- GroupInfoResponse
- message_utils_benchmark.rs
- PrivateKey
- SendResult
- VoiceRecorder
- bot.rs
- Vec
- ChainKey
- SenderKeyDeviceCache
- SecretBytes
- iq/business.rs
- MessageExt
- ResolvedGroupDevices
- assemble_status_participants
- waproto/src/lib.rs
- Downloadable
- Bot
- .issue_tokens
- DevicePropsOverride
- test_utils.rs
- .get_user_devices
- CurveError
- record_helpers.rs
- EnforcementExtraData
- zip.rs
- Server
- .update_local_signal_session
- state/session.rs
- prep.rs
- DownloadParams
- .send_message
- Jid
- EventRecorder
- enc_handler.rs
- GatewayImpl
- ExpandedAppStateKeys
- CameraSession
- Instant
- wacore
- Client
- Profile<'a>
- .sleep
- token.rs
- SignalProtocolError
- edge_routing.rs
- build_reaction_message
- ConversationExt
- protocol/keepalive.rs
- Device
- notification.rs
- create_record_with_previous_sessions
- .cast
- ImageCache
- PreparedVerifyingKey
- Client
- Client
- notification/profile.rs
- rust-backend/src/pair.rs
- InfoQuery
- .reader_from
- appstate_external_mutations_test.rs
- Phase 2 Frontend
- Device
- ProtocolNode
- .pair_with_code
- generate_reporting_token
- BenchRuntime
- Option
- Self
- PairUtils
- .spawn
- CacheStore
- ApiError
- measure_binary_size.py
- PendingDeviceSync
- scan_message_level
- prepare_peer_stanza
- wacore/src/store/error.rs
- src/reaction.rs
- EncPayload
- comment.rs
- .prepare_for_quote
- .poll
- LocalChatSettings
- waproto/build.rs
- binary_size_report.py
- notification/groups.rs
- .handle_sender_key_distribution_message
- history_sync_benchmark.rs
- Settings
- rust-backend/src/main.rs
- required_attr
- derive_reporting_token_key
- jid_benchmark.rs
- AttrsRef<'static>
- iq/keepalive.rs
- utils.rs
- .prepare_for_forward
- types/presence.rs
- klipy.rs
- rust-backend/src/version.rs
- PairCodeState
- attach.rs
- Labels<'a>
- .send_reaction
- bench_collect_unique_index_macs
- BenchRng
- CountingAlloc
- noise_benchmark.rs
- CoreClient
- Architecture & Process
- privacy_business.rs
- Phase 1 API surface & IPC mapping
- HandshakeUtils
- Schema
- CountingAlloc
- main
- NoiseCipher
- build_zero_signed_chain
- .into_base_message
- handlers/ib.rs
- Receipt
- handlers/presence.rs
- DisconnectReason
- build_ephemeral_setting_message
- abprops.rs
- MessageStanza
- release.sh
- regenerate-proto-desc.sh
- Recorder
- ConnectionOptions
- seed_peer_send_state
- ReceiptType
- paths.rs
- noise/src/handshake.rs
- Tauri desktop app (whatsvelte-rust)
- Tauri + Svelte → Slint migration plan
- PreKeyUploadIqNode
- slint-client/src/main.rs
- Protocol Architecture doc
- handlers/receipt.rs
- clock
- whatsapp-rust backend library
- .poll
- install_callbacks
- CachedServerCertChain
- TempBanReason
- .send_app_state_mutation

## God Nodes (most connected - your core abstractions)
1. `Result` - 1610 edges
2. `Response` - 166 edges
3. `ProtocolAddress` - 151 edges
4. `SqliteStore` - 112 edges
5. `create_test_client()` - 110 edges
6. `Event` - 93 edges
7. `MessageInfo` - 91 edges
8. `InMemoryBackend` - 83 edges
9. `InfoQuery` - 81 edges
10. `OwnedNodeRef` - 76 edges

## Surprising Connections (you probably didn't know these)
- `encode_query()` --references--> `String`  [EXTRACTED]
  slint-client/src/media/klipy.rs → rust-backend/wacore/binary/src/jid.rs
- `fmt_duration()` --references--> `String`  [EXTRACTED]
  slint-client/src/ui/views.rs → rust-backend/wacore/binary/src/jid.rs
- `Media & content types (asset protocol)` --conceptually_related_to--> `Tauri desktop app (whatsvelte-rust)`  [INFERRED]
  docs/phase-2-frontend.md → README.md
- `whatsapp-rust main client crate` --implements--> `whatsapp-rust backend library`  [INFERRED]
  rust-backend/AGENTS.md → README.md
- `whatsvelte-rust` --depends_on--> `whatsapp-rust`  [EXTRACTED]
  slint-client/Cargo.toml → rust-backend/Cargo.toml

## Import Cycles
- 1-file cycle: `rust-backend/wacore/libsignal/src/core/address.rs -> rust-backend/wacore/libsignal/src/core/address.rs`
- 2-file cycle: `rust-backend/wacore/libsignal/src/protocol/ratchet.rs -> rust-backend/wacore/libsignal/src/protocol/state/session.rs -> rust-backend/wacore/libsignal/src/protocol/ratchet.rs`

## Hyperedges (group relationships)
- **Rust backend agent docs set** — rust_backend_agents, rust_backend_agent_docs_protocol_architecture, rust_backend_agent_docs_feature_implementation, rust_backend_agent_docs_e2e_testing, rust_backend_agent_docs_debugging, rust_backend_agent_docs_binary_size_ci [EXTRACTED 1.00]
- **Tauri IPC contract documentation** — docs_architecture, docs_phase_1_api_surface, docs_phase_2_frontend [INFERRED 0.85]

## Communities (412 total, 7 thin omitted)

### Community 0 - "Result"
Cohesion: 0.02
Nodes (96): AtomicI32, BaseKeyKey, MsgSecretRow, test_cross_collection_mutations(), test_initial_sync_delivers_push_name(), test_multi_device_app_state_sync(), test_mutation_works_after_reconnect(), test_push_name_survives_reconnect() (+88 more)

### Community 1 - "SqliteStore"
Cohesion: 0.04
Nodes (61): CoreDevice, DieselError, batch_mutation_macs_matches_per_item(), clear_mutation_macs_wipes_only_named_collection(), create_test_store(), delete_expired_msg_secrets_deletes_only_passed_deadlines(), DeviceRow, DieselOrStore (+53 more)

### Community 2 - "Self"
Cohesion: 0.07
Nodes (21): AddParticipantsIq, BatchGetGroupInfoIq, GroupCreateOptions, GroupParticipantOptions, GroupParticipatingRequest, LinkedGroupResult, LinkSubgroupsResponse, MemberAddMode (+13 more)

### Community 3 - "mex_operations.rs"
Cohesion: 0.04
Nodes (190): Address, AdminProfile, AdminSettings, AimModelBatchedManifest, Appeal, Assets, Availability, BrandIdsData (+182 more)

### Community 4 - "message/tests.rs"
Cohesion: 0.07
Nodes (50): addon_decrypts_right_after_capture_without_flush(), app_state_sync_key_share_honored_only_from_self(), build_unavailable_stanza(), create_test_client_for_retry_with_id(), create_test_message_info(), enc_comment_inbound_dispatches_body_with_parent_link(), enc_reaction_inbound_decrypts_to_plaintext_shape(), ensure_bob_paired() (+42 more)

### Community 5 - "SignalStore"
Cohesion: 0.04
Nodes (19): msg_secret_batch_round_trip_and_overwrite(), AppStateSyncKey, AppSyncStore, DeviceStore, LidPnMappingEntry, merge_msg_secret_expiry(), merge_msg_secret_message_ts(), MsgSecretStore (+11 more)

### Community 6 - "SenderKeyName"
Cohesion: 0.05
Nodes (70): MemSenderKeyStore, MemSenderKeyStore, bench_decrypt_with_previous_session(), bench_dm_decrypt_first_message(), bench_dm_decrypt_subsequent_message(), bench_dm_encrypt_first_message(), bench_dm_encrypt_subsequent_message(), bench_dm_session_establishment() (+62 more)

### Community 7 - "rust-backend/src/send.rs"
Cohesion: 0.04
Nodes (63): InteractiveMessage, assert_biz_common_attrs(), assert_nested_biz(), BizCategory, build_edit_message(), build_edit_message_uses_top_level_protocol_message(), build_extra_stanza_nodes(), build_nested_biz() (+55 more)

### Community 8 - "iq/groups.rs"
Cohesion: 0.07
Nodes (60): NonZeroU32, AddRequestInfo, build_create_group_node(), GetGroupProfilePicturesIq, group_query_iq_without_phash_has_no_attr(), group_query_parse_full_vs_not_modified(), GroupQueryRequest, GroupQueryRequestType (+52 more)

### Community 9 - "AliceSignalProtocolParameters"
Cohesion: 0.12
Nodes (14): initialize_alice_session(), initialize_alice_session_record(), initialize_bob_session(), initialize_bob_session_record(), message_version(), AliceSignalProtocolParameters, BobSignalProtocolParameters, From (+6 more)

### Community 10 - "aes_gcm.rs"
Cohesion: 0.06
Nodes (63): Ctr32BE, GHash, MediaRetryNotification, Client, MediaReupload, MediaReupload<'a>, MediaReuploadRequest, Client (+55 more)

### Community 11 - "ProtocolAddress"
Cohesion: 0.04
Nodes (50): MemIdentityStore, MemSessionStore, HashMap, IdentityChange, SignalProtocolError, MemIdentityStore, IdentityChange, Self (+42 more)

### Community 12 - "MockBackend"
Cohesion: 0.06
Nodes (22): MockMacMap, build_patch_subtracts_previous_macs_fetched_in_batch(), create_encrypted_mutation(), MockBackend, process_patch_list_returns_snapshot_and_patches_to_caller(), Arc, AtomicU64, Bytes (+14 more)

### Community 13 - "protocol.rs"
Cohesion: 0.04
Nodes (27): Distribution, DecryptionErrorMessage, DecryptionErrorMessageProto, get_or_try_init_bytes(), PlaintextContent, PreKeySignalMessage, AsRef, Box (+19 more)

### Community 14 - "capturing_client"
Cohesion: 0.09
Nodes (53): bot_author_self_fanout_decrypt_failure_not_sender_receipt(), bot_reply_in_group_acks_with_bare_ack_not_receipt(), capturing_client(), collect_event(), custom_handler_only_skips_fallback_ack(), decrypt_failure_does_not_ack_when_retry_send_fails(), decrypt_failure_emits_transport_ack(), decrypted_message_edit_recaptures_secret_for_next_edit() (+45 more)

### Community 15 - "Event"
Cohesion: 0.09
Nodes (57): ArchiveChatAction, ClearChatAction, ContactAction, DeleteChatAction, DeleteMessageForMeAction, LabelAssociationAction, LabelEditAction, MarkChatAsReadAction (+49 more)

### Community 16 - "AttrParser<'a>"
Cohesion: 0.10
Nodes (9): AttrParser<'a>, AttrParserRef<'a>, coerce_protocol_bool(), Cow, Jid, Node, NodeRef, Option (+1 more)

### Community 17 - "text_msg"
Cohesion: 0.04
Nodes (61): EventResponseMessage, Message, text_msg(), test_expired_chatstate_not_delivered(), test_fresh_chatstate_delivered_on_reconnect(), concurrent_disconnect_three_clients(), concurrent_disconnect_two_clients_multithread(), concurrent_disconnect_two_clients_single_thread() (+53 more)

### Community 18 - "iq/usync.rs"
Cohesion: 0.15
Nodes (28): build_business_query_node(), build_test_key_index_list_node(), device_list_devices_error_skips_only_that_user(), device_list_result_devices_error_is_warn_only(), is_on_whatsapp_preserves_user_subprotocol_errors(), lid_query_lid_error_skips_only_that_user(), LidQuerySpec, parse_verified_name_skips_error_and_empty() (+20 more)

### Community 19 - "send_receive_benchmark.rs"
Cohesion: 0.10
Nodes (45): bench_dm_recv(), bench_dm_send(), bench_group_recv(), bench_group_send_10(), bench_group_send_256(), bench_group_send_50(), bench_group_send_skdm_10(), bench_group_send_skdm_256() (+37 more)

### Community 20 - "send/tests.rs"
Cohesion: 0.05
Nodes (48): build_member_label_message(), backfilled_wrappers_classify_by_inner(), build_member_label_message_clear_uses_empty_string(), build_member_label_message_preserves_unicode(), build_member_label_message_sets_fields(), detects_406_server_error_code(), ephemeral_wrapped_list_reaches_list_mediatype(), established_stores() (+40 more)

### Community 21 - "HashState"
Cohesion: 0.08
Nodes (36): FDownload, Mutation, SyncActionValue, HashState, Default, SyncdRecord, AppStateProcessor, AppStateSyncDriver (+28 more)

### Community 22 - "CryptoProviderError"
Cohesion: 0.07
Nodes (28): P, aes_256_gcm_decrypt(), aes_256_gcm_decrypt_in_place(), aes_256_gcm_encrypt(), aes_256_gcm_encrypt_in_place(), B, Box, Vec (+20 more)

### Community 23 - "Error"
Cohesion: 0.10
Nodes (15): Client, Groups<'a>, CompactString, GroupDescription, GroupSubject, Into, Jid, Self (+7 more)

### Community 24 - "src/receipt.rs"
Cohesion: 0.07
Nodes (65): aggregate_delivery_receipt_chunks_at_256_ids(), aggregate_delivery_receipt_node_shape_and_ingest_roundtrip(), aggregate_delivery_receipts_group_by_chat_author_and_type(), build_aggregate_delivery_receipt_nodes(), build_delivery_receipt_node(), build_nack_node(), build_played_receipt_node(), build_read_receipt_node() (+57 more)

### Community 25 - "rust-backend AGENTS guide"
Cohesion: 0.17
Nodes (16): Binary Size CI doc, Binary size PR gate (64 KiB stripped, 32 KiB .text), Debugging doc, evcxr REPL & nibble encoding, E2E Testing doc, E2E test patterns (event-driven waiting, reconnect_and_wait), Feature Implementation doc, rust-backend AGENTS guide (+8 more)

### Community 26 - "message_processing.rs"
Cohesion: 0.06
Nodes (47): Client, Client, Jid, Message, Node, Option, Self, Vec (+39 more)

### Community 27 - "stanza/call.rs"
Cohesion: 0.06
Nodes (65): CallHandler, fake_caller_lid(), make_client(), malformed_stanza_does_not_error_or_dispatch(), offer_dispatches_event(), offer_stanza(), offer_triggers_outbound_send(), Arc (+57 more)

### Community 28 - "crypto/hash.rs"
Cohesion: 0.06
Nodes (41): A, HmacReset, hkdf_sha256_into(), LTHash, perform_pointwise_with_overflow(), pointwise_add_and_subtract(), pre_keyed_extract_matches_plain_hkdf(), S (+33 more)

### Community 29 - "ServiceId"
Cohesion: 0.07
Nodes (30): Aci, KIND, Pni, append_device_suffix(), DeviceId, digit_count(), Debug, Display (+22 more)

### Community 30 - "client/tests.rs"
Cohesion: 0.04
Nodes (43): build_ack_node(), fibonacci_backoff(), message_ack_source_node(), Duration, cleanup_connection_state_flushes_dirty_sender_key(), cleanup_connection_state_flushes_dirty_signal_state(), cleanup_connection_state_keeps_state_when_flush_fails(), connect_failure_403_dispatches_account_locked_logout() (+35 more)

### Community 31 - "PreKeyId"
Cohesion: 0.07
Nodes (23): MemPreKeyStore, Option, PreKeyStore, SigResult, InMemoryPreKeyStore, HashMap, PreKeyStore, MemPreKeyStore (+15 more)

### Community 32 - "iq/tctoken.rs"
Cohesion: 0.07
Nodes (45): bucket_index(), build_cs_token_node(), build_tc_token_node(), build_tc_token_node_with_timestamp(), compute_cs_token(), expiration_cutoff_at(), is_sender_tc_token_expired(), is_tc_token_expired() (+37 more)

### Community 33 - "iq/prekeys.rs"
Cohesion: 0.07
Nodes (39): required_child(), assert_direct_encode_matches_marshal(), DigestKeyBundleResponse, expand_from_3bytes(), OneTimePreKeyNode, PreKeyBundleUserNode, PreKeyCountResponse, PreKeyCountSpec (+31 more)

### Community 34 - "Vec"
Cohesion: 0.20
Nodes (15): ik_handshake_round_trip_continue(), ik_serve_accept(), ik_serve_force_fallback(), ik_to_xx_fallback_round_trip(), IkHandshakeState, NoiseHandshake, process_xx_server_hello_into(), KeyPair (+7 more)

### Community 35 - "session_cipher.rs"
Cohesion: 0.08
Nodes (50): CiphertextMessage, CiphertextMessageType, CurrentOrPrevious, decrypt_corrupted_mac_returns_bad_mac(), decrypt_message_with_record(), decrypt_message_with_state(), decrypt_prekey_reports_identity_change(), decrypt_with_empty_session_returns_session_not_found() (+42 more)

### Community 36 - "wacore/src/upload.rs"
Cohesion: 0.13
Nodes (29): MediaType, bytes_upload_source_matches_arc_slice(), bytes_upload_source_reads_from_offset(), encrypt_media(), encrypt_media_streaming(), encrypt_media_streaming_with_key(), encrypt_media_with_key(), encrypt_media_with_key_and_sidecar() (+21 more)

### Community 37 - "messages.rs"
Cohesion: 0.09
Nodes (46): assert_splice_matches(), broadcast_populates_bcl_participants(), context_test_shapes(), decode_padded(), decode_plaintext(), DmPlaintexts, envelope_enrichment_fields_are_captured(), envelope_enrichment_is_optional() (+38 more)

### Community 38 - "src/node.rs"
Cohesion: 0.11
Nodes (19): AttrsVec, Attrs, bytes_content_serializes_same(), BytesCart, intern_cow(), nested_nodes_serialize_same(), node_ref_serializes_same_as_node(), NodeValue (+11 more)

### Community 39 - "blocklist.rs"
Cohesion: 0.08
Nodes (30): Blocking, Blocking<'a>, blocklist_contains(), Client, lid_entry(), Client, IqError, Jid (+22 more)

### Community 40 - "wacore/src/download.rs"
Cohesion: 0.12
Nodes (18): AesCbcDecryptionError, CryptoError, copy_and_validate_plaintext_mismatch(), copy_and_validate_plaintext_ok(), DownloadUtils, MediaConnection, MediaDecryption, MediaDecryptionError (+10 more)

### Community 41 - "client/lid_pn.rs"
Cohesion: 0.09
Nodes (36): Client, learn_fast_offline_then_live_persists(), mapping_to_entry(), migrate_skips_when_no_pn_signal_state(), migration_blocks_on_per_address_session_lock(), migration_lock_dance_completes_when_caller_drops_guard(), migration_preserves_working_session_when_both_namespaces_present(), migration_reports_whether_anything_moved() (+28 more)

### Community 42 - "features/message_edit.rs"
Cohesion: 0.09
Nodes (43): decrypt(), decrypt_normalises_device_suffix(), decrypt_secret_encrypted(), decrypt_secret_encrypted_roundtrip_poll_edit(), decrypt_secret_encrypted_with_fallback(), decrypt_with_fallback(), edit_author_from_envelope(), enc_comment_decrypts_to_inner_body() (+35 more)

### Community 43 - "create_test_client"
Cohesion: 0.11
Nodes (53): create_test_client(), get_devices_from_registry_reads_empty_record_as_miss(), group_devices_memo_hits_and_invalidates(), group_devices_memo_invalidated_by_member_mapping_change(), group_devices_memo_scoped_invalidation(), lid_keyed_update_invalidates_pn_group_memo(), make_device_element(), make_signed_key_index_bytes() (+45 more)

### Community 44 - "wacore/src/history_sync.rs"
Cohesion: 0.11
Nodes (50): differential_fast_path_matches_prost_oracle(), emit_len_field(), emit_varint(), encode_and_compress(), extraction_reports_exact_decompressed_size(), FieldIter, FieldWalker, fp() (+42 more)

### Community 45 - "rust-backend/src/download.rs"
Cohesion: 0.09
Nodes (27): ExecuteRequest, InvalidateMediaConn, PrepareRequests, CannedHttpClient, Client, download_media_with_retry(), download_retries_with_forced_media_conn_refresh_after_auth_error(), download_to_writer_retries_with_forced_media_conn_refresh_after_auth_error() (+19 more)

### Community 46 - "proto_helpers.rs"
Cohesion: 0.05
Nodes (25): build_quote_context_with_info(), create_message_with_mentions(), merge_dsm_context(), quote_context_cross_chat_remote_jid_drops_device_suffix(), quote_context_device_suffix_treated_as_same_chat(), quote_context_emits_remote_jid_cross_chat(), quote_context_newsletter_cross_chat_sets_both(), quote_context_newsletter_uses_channel_as_participant() (+17 more)

### Community 47 - "stanza/groups.rs"
Cohesion: 0.11
Nodes (49): admin_jid(), group_jid(), GroupNotification, GroupNotificationAction, GroupParticipantInfo, GroupParticipantType, make_notification(), MembershipRequestMethod (+41 more)

### Community 48 - "sticker_pack.rs"
Cohesion: 0.08
Nodes (33): base64url_encode(), build_message_fields(), build_sticker_pack_message(), create_sticker_pack_zip(), create_zip_basic(), create_zip_deduplication(), create_zip_empty_rejected(), create_zip_too_many_rejected() (+25 more)

### Community 49 - "Runtime"
Cohesion: 0.09
Nodes (32): AbortHandle, blocking(), captured_signal_observes_fire_after_notifier_dropped(), Elapsed, notifier_listen_catches_notify_fired_before_listen(), Arc, AtomicBool, Box (+24 more)

### Community 50 - "NodeStr"
Cohesion: 0.06
Nodes (30): FromIterator, &'a Attrs, _assert_attrs_ref_covariant(), AttrsRef, AttrsRef<'a>, AttrsRefWrapper, NodeContentRef, NodeStr (+22 more)

### Community 51 - "expand_app_state_keys"
Cohesion: 0.13
Nodes (35): G, AppStateError, generate_index_mac(), validate_index_mac(), expand_app_state_keys(), expansion_deterministic(), create_encrypted_record(), detect_duplicate_index_in_patch() (+27 more)

### Community 52 - "send_and_expect_text"
Cohesion: 0.08
Nodes (48): restricted_push_name(), Client, scenario_push_name(), send_and_expect_text(), assert_lid_only_sessions(), mask_addr(), Vec, scan_sessions() (+40 more)

### Community 53 - "binary_benchmark.rs"
Cohesion: 0.06
Nodes (87): bench_attr_parser(), bench_get_children_by_tag(), bench_jid_to_owned_access(), bench_marshal_auto_huge_bytes(), bench_marshal_auto_large(), bench_marshal_auto_long_string(), bench_marshal_auto_many_children(), bench_marshal_auto_small() (+79 more)

### Community 54 - "GenericSignedPreKey"
Cohesion: 0.11
Nodes (13): Id, GenericSignedPreKey, KeyPair, KeyPairSerde, KeySerde, PrivateKey, PublicKey, KeyPair (+5 more)

### Community 55 - "PreKeyBundle"
Cohesion: 0.05
Nodes (32): MemSignedPreKeyStore, SignedPreKeyStore, InMemorySignedPreKeyStore, SignedPreKeyStore, PreKeyBundle, PreKeyBundleContent, F, From (+24 more)

### Community 56 - "MexError"
Cohesion: 0.12
Nodes (27): MexError, Client, mute_user_setting_variables(), mute_variables_match_wa_web_shape(), Newsletter, Newsletter<'a>, NewsletterMessage, NewsletterMessageType (+19 more)

### Community 57 - "Device"
Cohesion: 0.11
Nodes (18): Device, DeviceRwLockWrapper, DeviceStore, Arc, Clone, IdentityChange, Mutex, Option (+10 more)

### Community 58 - "privacy.rs"
Cohesion: 0.13
Nodes (13): attr_str(), PrivacySettingsSpec, Cow, Node, NodeRef, SetDefaultDisappearingModeSpec, test_parse_all_categories_from_server(), test_privacy_settings_spec_build_iq() (+5 more)

### Community 59 - "SenderKeyState"
Cohesion: 0.06
Nodes (40): backlog_mutation_after_clone_is_isolated(), InvalidSenderKeySessionError, Arc, Debug, Display, ExactSizeIterator, Formatter, Item (+32 more)

### Community 60 - "SignalStoreCache"
Cohesion: 0.09
Nodes (33): addr(), ByteStoreState, checked_out_session_defers_prekey_delete_until_durable(), checked_out_sessions_are_never_evicted(), clear_before_flush_keeps_prekey_so_pkmsg_can_rebuild(), concurrent_decrypt_does_not_lose_prekey_during_flush(), consumed_prekey_stays_durable_until_session_flush(), dirty_entries_are_never_evicted() (+25 more)

### Community 61 - "rust-backend/src/handshake.rs"
Cohesion: 0.11
Nodes (33): CoreHandshakeError, cached_chain(), do_handshake(), HandshakeError, HandshakePattern, HandshakeSuccess, paired_device(), recv_frame() (+25 more)

### Community 62 - "encrypt_bot_message"
Cohesion: 0.27
Nodes (21): BotMessageContext, build_aad(), build_aad_matches_wa_web_layout(), decrypt_bot_message(), decrypt_rejects_ciphertext_tampering(), decrypt_rejects_short_iv(), decrypt_rejects_short_payload(), decrypt_rejects_tag_tampering() (+13 more)

### Community 63 - "rust-backend/src/prekeys.rs"
Cohesion: 0.09
Nodes (28): backend(), clamp_wanted_pre_key_count(), Client, consumed_head_advances_to_next_live_window_key(), failed_upload_abandons_window_and_never_remints_ids(), fully_missing_window_collapses_and_regenerates_in_one_pass(), plan_collapses_window_at_id_boundary(), plan_full_window_generates_nothing_and_keeps_next() (+20 more)

### Community 64 - "src/retry.rs"
Cohesion: 0.08
Nodes (17): make_test_receipt(), resolve_retry_chat_info(), resolve_retry_chat_info_bot_with_recipient(), resolve_retry_chat_info_bot_without_recipient(), resolve_retry_chat_info_dm_bare(), resolve_retry_chat_info_dm_with_device(), resolve_retry_chat_info_fbid_bot_dm_marks_bot_retry(), resolve_retry_chat_info_forwards_recipient_attribute_verbatim() (+9 more)

### Community 65 - "Client"
Cohesion: 0.13
Nodes (16): Client, dm_sender_identity_picks_lid_for_bot_else_pn(), ensure_self_in_group(), is_self_dm_recipient(), persist_outbound_msg_secret_writes_under_chat_sender_id(), persist_uses_group_sender_identity_for_lid_mode_groups(), RevokeType, Arc (+8 more)

### Community 66 - "ratchet/keys.rs"
Cohesion: 0.14
Nodes (19): derive_keys_no_salt_matches_plain_hkdf(), MessageKeyGenerator, MessageKeys, MessageKey, Option, Self, test_chain_key_long_chain(), test_chain_key_overflow_boundary() (+11 more)

### Community 67 - "props.rs"
Cohesion: 0.10
Nodes (24): optional_attr(), AbProp, AbPropConfig, PropsResponse, PropsSpec, CompactString, Into, Node (+16 more)

### Community 68 - "SessionRecord"
Cohesion: 0.16
Nodes (6): Arc, Box, Option, Vec, SessionRecord, SignalProtocolError

### Community 69 - "Client"
Cohesion: 0.10
Nodes (38): ChatStateHandler, build_pong(), ChatLane, Client, encode_ack_bytes(), is_encrypt_identity_notification(), NodeFilter, NodeWaiter (+30 more)

### Community 70 - "notification/mod.rs"
Cohesion: 0.11
Nodes (31): handle_notification_impl(), node_to_arc(), NotificationHandler, parse_disappearing_mode(), Arc, Client, Node, Option (+23 more)

### Community 71 - "signal_adapter.rs"
Cohesion: 0.11
Nodes (23): buffer_consumed_prekey_defers_backend_delete_to_flush(), IdentityAdapter, PreKeyAdapter, remove_pre_key_deletes_immediately(), Arc, Device, E, FnOnce (+15 more)

### Community 72 - "state.rs"
Cohesion: 0.14
Nodes (18): generate_iv(), NoiseBuffer, NoiseKeys, NoiseState, AsRef, Option, Vec, sha256_digest() (+10 more)

### Community 73 - "StanzaHandler"
Cohesion: 0.06
Nodes (30): AckHandler, FailureHandler, Arc, Client, StreamErrorHandler, SuccessHandler, IqHandler, Arc (+22 more)

### Community 74 - ".new"
Cohesion: 0.13
Nodes (18): aggregate_interest_and_has_handler_for(), ChannelEventHandler, Connected, CoreEventBus, dispatch_is_reentrancy_safe_against_concurrent_add(), dispatch_preserves_handler_ordering(), dispatch_respects_dynamically_widened_interest(), EventHandler (+10 more)

### Community 75 - "Arc"
Cohesion: 0.18
Nodes (33): AlicePeer, assert_exactly_one_confirmation(), await_retry_receipt(), bad_session_plaintext_skips_skmsg_sibling_after_nack(), bobs_prekey_bundle(), duplicate_message_is_acked_with_delivery_receipt(), enc_payload_from_ciphertext(), group_message_info() (+25 more)

### Community 76 - "framing.rs"
Cohesion: 0.11
Nodes (22): BytesMut, encode_frame(), encode_frame_into(), FrameBuf, FrameDecoder, Bytes, Default, Option (+14 more)

### Community 77 - "GroupInfo"
Cohesion: 0.15
Nodes (24): add_participants_backfills_lid_map_for_existing(), add_participants_deduplicates(), add_participants_lid_mode_updates_maps(), add_participants_pn_mode(), build_pn_to_lid_map(), GroupInfo, GroupInfoDe, lid() (+16 more)

### Community 78 - "Store"
Cohesion: 0.07
Nodes (19): add_dedupes_and_bumps_unread(), ChatMsgs, is_blank(), learning_a_lid_merges_split_chats(), MessageStatus, msg(), optimistic_is_confirmed_and_receipts_never_downgrade(), revoke_of_unknown_message_inserts_tombstone() (+11 more)

### Community 79 - "PortableCache<K, V>"
Cohesion: 0.16
Nodes (20): AsyncMutex, CacheEntry, CacheInner, CacheInner<K, V>, PortableCache, PortableCache<K, V>, PortableCacheBuilder, Arc (+12 more)

### Community 80 - "store/device.rs"
Cohesion: 0.19
Nodes (22): android_profile_emits_android_payload_without_web_info(), android_profile_survives_login_payload_path(), client_profile_independent_of_device_props_platform_type(), default_profile_emits_legacy_web_payload(), every_native_profile_drops_web_info_in_payload(), history_sync_config_override_reaches_registration_payload(), login_counter_survives_serde_roundtrip(), login_payload_has_no_device_props() (+14 more)

### Community 81 - "iq/contacts.rs"
Cohesion: 0.14
Nodes (22): for_group_routes_empty_to_remove(), ProfilePicture, ProfilePictureSpec, ProfilePictureType, Jid, NodeRef, Option, Self (+14 more)

### Community 82 - "features/presence.rs"
Cohesion: 0.13
Nodes (21): Presence, Client, create_test_backend(), Presence, Presence<'a>, PresenceError, PresenceStatus, Arc (+13 more)

### Community 83 - "handshake_integration.rs"
Cohesion: 0.20
Nodes (31): CaptureTransport, cold_start_xx_then_cached_ik_reconnect(), ik_continue_does_not_overwrite_cached_chain(), ik_rejected_recovers_via_xxfallback_and_repopulates_cache(), ik_serve_accept(), ik_serve_fallback_with_corrupt_payloads(), ik_serve_force_fallback_then_consume_finish(), ik_with_stale_cache_invalidates_and_increments_counter() (+23 more)

### Community 84 - ".fmt"
Cohesion: 0.18
Nodes (8): JidStackWriter, Err, Formatter, Ok, S, W, Write, write_jid_fallible()

### Community 85 - "wacore/src/pair_code.rs"
Cohesion: 0.10
Nodes (26): android_device_props_emit_server_accepted_companion_hello(), build_iq(), child_bytes(), companion_hello_iq_chrome_linux_wire_parity(), companion_hello_iq_passes_through_explicit_android_letter(), companion_hello_iq_shape(), derive_android_platform_types_map_to_chrome(), derive_chrome_linux_matches_wa_web() (+18 more)

### Community 86 - "MockSendContextResolver"
Cohesion: 0.12
Nodes (21): ChainLockProbe, create_mock_bundle(), encrypt_for_devices_reports_replaced_identity(), MemSessionStore, MockSendContextResolver, prekey_fetch_runs_outside_chain_lock(), Arc, AtomicBool (+13 more)

### Community 87 - "String"
Cohesion: 0.07
Nodes (57): String, Code, Collection, Collections, ComplianceInfo, Content, Creatives, CustomUrl (+49 more)

### Community 88 - "bridge.rs"
Cohesion: 0.11
Nodes (47): album_parent_of(), BridgeHandler, chat_flags_dto(), classify(), context_info_of(), convert_conversation(), is_renderable(), media_dto() (+39 more)

### Community 89 - "BinaryError"
Cohesion: 0.08
Nodes (18): EncryptSendError, EncryptSendErrorKind, Into, Self, SocketError, BinaryError, Clone, Display (+10 more)

### Community 90 - "OwnedNodeRef"
Cohesion: 0.10
Nodes (11): AttrParser, AttrParserRef, Vec, Node, NodeRef, NodeRef<'a>, OwnedNodeRef, Formatter (+3 more)

### Community 91 - "NodeContent"
Cohesion: 0.13
Nodes (21): NodeContent, Node, InfoQuery<'a>, InfoQueryType, IqError, parse_iq_response_accepts_result_type(), parse_iq_response_error_without_backoff_is_none(), parse_iq_response_extracts_error_type_and_backoff() (+13 more)

### Community 92 - "session_divergence.rs"
Cohesion: 0.31
Nodes (21): alice_delete_then_rebuild_loses_old_chain(), alice_loses_session_entirely_yields_session_not_found(), alice_rebuilds_session_repeatedly_old_chain_still_decrypts(), baseline_dm_ping_pong(), deep_chain_survives_repeat_rebuilds(), dh_ratchet_step_preserves_decryption(), establish(), failed_mac_must_not_advance_receiver_chain() (+13 more)

### Community 93 - "stanza/devices.rs"
Cohesion: 0.11
Nodes (21): DeviceElement, DeviceNotification, DeviceOperation, KeyIndexInfo, Jid, Node, NodeRef, Option (+13 more)

### Community 94 - ".new"
Cohesion: 0.27
Nodes (25): C, DefaultHttpState, DefaultRuntimeState, DefaultTransportState, Rt, BotBuilder<MissingBackend, DefaultTransportState, DefaultHttpState, DefaultRuntimeState>, create_test_sqlite_backend(), from_arc_does_not_deep_clone() (+17 more)

### Community 95 - "src/message.rs"
Cohesion: 0.14
Nodes (22): MutexGuardArc, ClassifiedMessage, is_sender_key_distribution_only(), MigrationDecryptOutcome, PlaintextHandleOutcome, Client, Arc, Jid (+14 more)

### Community 96 - ".new"
Cohesion: 0.17
Nodes (25): N, Encoder<'static, IoByteWriter<W>>, TestResult, test_ad_jid_domain_type_lid(), test_ad_jid_domain_type_whatsapp(), test_ad_jid_domain_type_whatsapp_ignores_hidden_agent(), test_binary_length_boundaries(), test_bot_jid_with_device_round_trips_via_jid_pair() (+17 more)

### Community 97 - "pdo.rs"
Cohesion: 0.15
Nodes (27): PlaceholderMessageResendResponse, Client, empty_device(), make_group_message_info(), make_web_msg(), pdo_missing_content_response_clears_pending_but_keeps_memo(), pdo_request_failure_releases_once_per_message_slot(), pdo_request_skipped_when_already_requested() (+19 more)

### Community 98 - "Jid"
Cohesion: 0.06
Nodes (14): AcceptGroupInviteV4Iq, AcknowledgeGroupIq, DeleteCommunityIq, GetGroupInviteLinkIq, GetLinkedGroupsParticipantsIq, GetMembershipRequestsIq, JoinGroupResult, LeaveGroupIq (+6 more)

### Community 99 - "polls.rs"
Cohesion: 0.14
Nodes (22): PollCreationMessage, aggregate_clears_vote_across_namespace(), aggregate_dedupes_revote_across_namespace(), aggregate_votes_recovers_across_addressing(), build_poll_creation_message(), Client, decrypt_vote_fails_without_mapping(), decrypt_vote_recovers_when_fed_lid_but_encrypted_under_pn() (+14 more)

### Community 100 - "LidPnCache"
Cohesion: 0.11
Nodes (16): LidPnCache, persisted_marker_is_pair_specific(), Arc, CompactString, Default, IntoIterator, Item, OnceLock (+8 more)

### Community 101 - "SessionManager"
Cohesion: 0.11
Nodes (29): MediaDescriptorIn, Api, b64_decode(), download_permits(), ensure_cached(), ext_for(), media_type_of(), MediaSendOptionsIn (+21 more)

### Community 102 - "companion_reg.rs"
Cohesion: 0.07
Nodes (14): companion_browser_name(), companion_platform_display(), companion_web_client_type_for_platform(), CompanionWebClientType, Display, Formatter, PlatformType, build_id_and_display() (+6 more)

### Community 103 - "prepare_dm_retry_stanza"
Cohesion: 0.16
Nodes (25): extract_ciphertext(), media_type_from_message(), peer_message_options_from_message(), Box, Message, Option, should_hide_decrypt_fail(), should_hide_decrypt_fail_for_send() (+17 more)

### Community 104 - "TestClient"
Cohesion: 0.13
Nodes (14): mock_admin_scan_qr_url(), mock_server_url(), Arc, F, Jid, JoinHandle, Node, Option (+6 more)

### Community 105 - "src/msg_secret.rs"
Cohesion: 0.11
Nodes (24): classify(), classify_from_flags(), expires_at(), full_and_disabled_never_expire(), invokes_bot(), is_bot_context(), is_bot_context_detects_chat_and_invocation(), managed_text_expires_30d_after_message_time() (+16 more)

### Community 106 - "Controller"
Cohesion: 0.10
Nodes (7): Controller, Arc, FnOnce, Option, Self, Settings, normalize_jid()

### Community 107 - "MsgSecretWriteBuffer"
Cohesion: 0.16
Nodes (21): Key, buffer(), burst_coalesces_into_one_batch(), coalesced_duplicates_merge_retention_metadata(), entries_queued_during_drain_are_flushed(), entry(), explicit_flush_drains_everything(), metadata_refresh_during_flush_survives_removal() (+13 more)

### Community 108 - "sessions.rs"
Cohesion: 0.09
Nodes (4): Client, Duration, Jid, Vec

### Community 109 - ".new"
Cohesion: 0.14
Nodes (36): batch_accumulates_undecryptable_and_dispatches_once(), bot_only_captures_group_bot_prompt_skips_plain(), mock_http_client(), mock_transport(), Self, skdm_only_detection_restores_carrier_fields(), test_empty_session_record_treated_as_session_not_found(), test_handle_incoming_message_skips_skmsg_after_msg_failure() (+28 more)

### Community 110 - "encrypt.rs"
Cohesion: 0.17
Nodes (28): CloneableIdentityStore, CloneableSessionStore, encrypt_for_devices(), encrypt_for_devices_with_sessions(), encrypt_one_device(), EncryptOneResult, EncryptResult, ensure_sessions_for_devices() (+20 more)

### Community 111 - "Jid"
Cohesion: 0.13
Nodes (8): Jid, CompactString, D, From, Into, Self, test_jid_factories_with_string_types(), test_jid_to_non_ad_preserves_user_server()

### Community 112 - "LazyHistorySync"
Cohesion: 0.10
Nodes (22): lazy_from(), lazy_history_sync_caches_decode(), lazy_history_sync_clone_is_cheap_and_redecodes(), lazy_history_sync_decompress_yields_raw_proto(), lazy_history_sync_empty_proto_decodes_default(), lazy_history_sync_everything_keeps_working_after_get(), lazy_history_sync_get_decodes(), lazy_history_sync_preserves_messages() (+14 more)

### Community 113 - "rust-backend/src/history_sync.rs"
Cohesion: 0.16
Nodes (26): HistorySyncMsg, Client, compress_history_sync(), group_history_msg(), history_msg(), history_msg_secret_senders(), history_notification(), history_seed_botonly_keeps_group_bot_prompt_skips_plain() (+18 more)

### Community 114 - "Client"
Cohesion: 0.09
Nodes (9): Client, Arc, GroupCache, Jid, Node, Option, PersistenceManager, Receiver (+1 more)

### Community 115 - "Client"
Cohesion: 0.14
Nodes (10): Client, GroupDevicesMemo, CompactString, HashSet, Jid, Option, StoreError, Vec (+2 more)

### Community 116 - "Client"
Cohesion: 0.18
Nodes (8): Client, Arc, BoxFuture, Duration, Option, PersistenceManager, Receiver, Self

### Community 117 - ".drop"
Cohesion: 0.14
Nodes (15): abort_handle_in_arc_drops_only_when_last_ref_released(), PersistenceManager, Arc, AtomicBool, Device, Duration, F, Option (+7 more)

### Community 118 - ".handle_receipt"
Cohesion: 0.16
Nodes (17): Client, node_to_arc(), offline_receipt_buffer_protocol(), Arc, Option, Self, setup_client_with_collector(), test_aggregated_by_message_receipt_fans_out_per_user() (+9 more)

### Community 119 - "BenchResults"
Cohesion: 0.14
Nodes (19): AllocDelta, AllocSnapshot, CountingAlloc, GlobalAlloc, Layout, Self, bench_connect_to_ready(), bench_receive_message() (+11 more)

### Community 120 - "decoder.rs"
Cohesion: 0.10
Nodes (32): Decoder, Decoder<'a>, jid_ref_to_compact(), CompactString, NodeRef, Option, Self, TestResult (+24 more)

### Community 121 - "encoder.rs"
Cohesion: 0.16
Nodes (26): binary_len_prefix_size(), build_marshaled_node_plan(), build_marshaled_node_ref_plan(), bytes_with_len_encoded_size(), classify_string_hint(), jid_ref_encoded_size_with_cache(), list_start_encoded_size(), MarshaledSizePlan (+18 more)

### Community 122 - "Controller"
Cohesion: 0.16
Nodes (18): AtomicU8, Handle, Notify, Controller, lock(), Api, AppWindow, Arc (+10 more)

### Community 123 - "adv.rs"
Cohesion: 0.11
Nodes (26): adv_chain_hosted_prefix_accepted(), adv_chain_in_blob_key_takes_precedence_over_fallback(), adv_chain_missing_account_key_no_fallback_is_no_account_key(), adv_chain_missing_account_key_verifies_with_fallback(), adv_chain_missing_account_key_wrong_fallback_is_invalid(), adv_chain_rejects_substituted_identity(), adv_chain_valid_accepted(), AdvValidation (+18 more)

### Community 124 - "wacore/src/pair.rs"
Cohesion: 0.16
Nodes (26): auto_derive_default_device_props_yields_other_web_client_nine(), auto_derive_from_device_props_round_trip(), build_pair_success_payload(), DeviceState, do_pair_crypto_accepts_matching_hmac(), do_pair_crypto_accepts_matching_hmac_for_hosted_account(), do_pair_crypto_rejects_mismatched_hmac(), do_pair_crypto_rejects_mismatched_hmac_for_hosted_account() (+18 more)

### Community 125 - "src/time.rs"
Cohesion: 0.13
Nodes (25): ChronoTimeProvider, default_monotonic_provider(), default_time_provider(), from_millis(), from_millis_or_now(), from_secs(), from_secs_or_now(), MonotonicProvider (+17 more)

### Community 126 - "types/message.rs"
Cohesion: 0.15
Nodes (20): BotEditType, infer_from_message_admin_revoke(), infer_from_message_legacy_edit(), infer_from_message_message_edit_sender(), infer_from_message_plain_returns_none(), infer_from_message_sender_revoke(), infer_from_message_top_level_edit(), infer_from_message_unwraps_neutral_wrappers() (+12 more)

### Community 127 - "rust-backend/src/upload.rs"
Cohesion: 0.13
Nodes (24): EXR, GMC, IMC, arc_upload_source_reads_from_offset_repeatably(), build_resume_check_request(), build_upload_request(), crypto_from(), media_conn() (+16 more)

### Community 128 - "features/community.rs"
Cohesion: 0.15
Nodes (18): Client, Community, Community<'a>, CommunitySubgroup, CreateCommunityOptions, CreateCommunityResult, group_type(), GroupType (+10 more)

### Community 129 - "types/spam_report.rs"
Cohesion: 0.11
Nodes (22): Client, IqError, test_build_spam_list_node_basic(), test_build_spam_list_node_group(), test_build_spam_list_node_with_raw_message(), NodeRef, Self, SpamReportSpec (+14 more)

### Community 130 - "tests/media.rs"
Cohesion: 0.10
Nodes (29): build_audio_message(), build_document_message(), build_image_message(), build_video_message(), From, Message, Option, Self (+21 more)

### Community 132 - "frame.rs"
Cohesion: 0.13
Nodes (33): AlbumTile, AppState, ChatRow, Color, Image, ModelRc, MsgRow, Rc (+25 more)

### Community 133 - "Controller"
Cohesion: 0.15
Nodes (7): Controller, rgba_pixels(), Arc, DynamicImage, PathBuf, Pixels, Self

### Community 134 - "derive/src/lib.rs"
Cohesion: 0.17
Nodes (29): Attribute, Field, Fields, Ident, Punctuated, AttrFieldInfo, AttrType, derive_empty_node() (+21 more)

### Community 135 - "zlib_pool.rs"
Cohesion: 0.16
Nodes (20): Decompress, decompress_zlib_pooled(), drain_reader(), drop_shrinks_oversized_buffer_before_pooling(), grow_by_observed_ratio(), inflate_reader_enforces_max(), inflate_reader_ensure_larger_than_chunk(), inflate_reader_reuse_after_error() (+12 more)

### Community 136 - "Api"
Cohesion: 0.15
Nodes (9): Api, parse_jid(), ApiResult, Arc, Jid, Option, Self, Vec (+1 more)

### Community 137 - "iq/mediaconn.rs"
Cohesion: 0.15
Nodes (16): HostType, MediaConnHost, MediaConnHostExtended, MediaConnResponse, MediaConnResponseExtended, MediaConnSpec, Node, NodeRef (+8 more)

### Community 138 - "types/jid.rs"
Cohesion: 0.11
Nodes (17): cmp_for_lock_order(), Jid, JidExt, make_address_buffer(), make_reusable_protocol_address(), mapped_server(), observe_protocol_address(), Jid (+9 more)

### Community 139 - "views.rs"
Cohesion: 0.12
Nodes (23): media_label(), quoted_text(), album_len(), album_rules(), albumable(), avatar_initials(), Controller, find_links() (+15 more)

### Community 140 - "AbPropsCache"
Cohesion: 0.15
Nodes (18): AbProp, AbPropsCache, default_interest_retains_production_flags(), delta_merges_without_clearing(), flag(), is_enabled_checks_truthy_values(), AtomicBool, CompactString (+10 more)

### Community 141 - "InvalidSessionError"
Cohesion: 0.14
Nodes (7): InvalidSessionError, Display, Formatter, PrivateKey, PublicKey, Self, UnacknowledgedPreKeyMessageItems

### Community 142 - "UnifiedSessionManager"
Cohesion: 0.14
Nodes (18): AtomicI64, test_unified_session_id_calculation(), Arc, AtomicU64, Default, Mutex, Node, NodeRef (+10 more)

### Community 143 - "TokioWebSocketTransportFactory"
Cohesion: 0.14
Nodes (20): Connector, default_tls_connector(), from_websocket(), read_pump(), Arc, Bytes, Default, Into (+12 more)

### Community 144 - "HttpClient"
Cohesion: 0.09
Nodes (19): MockHttpClient, MockHttpClient, FailingMockHttpClient, MockHttpClient, unreachable_check(), upload_error_from_response(), HttpClient, HttpRequest (+11 more)

### Community 145 - "iq/chatstate.rs"
Cohesion: 0.12
Nodes (14): ChatstateParseError, ChatstateSource, ChatstateStanza, ReceivedChatState, Jid, Node, NodeRef, Self (+6 more)

### Community 146 - "GroupMetadata"
Cohesion: 0.11
Nodes (19): BatchGroupResult, CreateGroupResult, extract_code_param(), extract_invite_code(), GroupMetadata, GroupParticipant, GroupPropertyUpdate, Groups (+11 more)

### Community 147 - "portable_cache.rs"
Cohesion: 0.17
Nodes (22): build_cache(), PortableCacheBuilder<K, V>, Duration, Self, test_basic_insert_and_get(), test_capacity_eviction(), test_get_with_basic(), test_get_with_by_ref_basic() (+14 more)

### Community 148 - "discrepancy_pocs.rs"
Cohesion: 0.09
Nodes (14): build_media_payload(), regression_a10_media_decrypt_accepts_valid_mac_roundtrip(), regression_a10_media_decrypt_rejects_mac_flipped_at_byte_zero(), regression_a10_media_decrypt_rejects_mac_flipped_at_last_byte(), regression_a7_content_mac_matches_wa_web_at_short_key_id(), regression_a7_content_mac_matches_wa_web_at_wrap_boundary(), SyncdOperation, Vec (+6 more)

### Community 149 - "src/jid.rs"
Cohesion: 0.10
Nodes (18): assert_jid_parse_and_display(), assert_jid_roundtrip(), observe_redacts_phone_but_not_lid_or_group(), observe_str(), observe_token(), ObservedJid, parse_jid_fast(), ParsedJidParts (+10 more)

### Community 150 - "features/mex.rs"
Cohesion: 0.06
Nodes (25): Client, Mex, Mex<'a>, MexRequest, MexRequest<V>, Client, IqError, Self (+17 more)

### Community 151 - "Filters"
Cohesion: 0.36
Nodes (10): Clauses, Clauses2, Clauses3, Clauses4, Clauses5, Clauses6, Clauses7, ContextualFiltersForWaDoNotUse (+2 more)

### Community 152 - "stanza/business.rs"
Cohesion: 0.15
Nodes (21): BusinessNotification, BusinessNotificationType, BusinessSubscription, Jid, NodeRef, Option, Self, Vec (+13 more)

### Community 153 - "dirty.rs"
Cohesion: 0.15
Nodes (17): IqError, CleanDirtyBitsSpec, DirtyBit, DirtyBitParseError, DirtyType, Into, NodeRef, Option (+9 more)

### Community 154 - "Jid"
Cohesion: 0.26
Nodes (5): ChatActions<'a>, message_key_owned(), Jid, Option, SyncActionMessageRange

### Community 155 - "protocol/retry.rs"
Cohesion: 0.10
Nodes (9): build_retry_keys_node(), extract_registration_id_from_node(), get_bytes_content(), retry_keys_bundle_emits_raw_32_byte_curve_values(), Node, Option, PublicKey, Vec (+1 more)

### Community 156 - ".write_string"
Cohesion: 0.12
Nodes (8): EncodeNode, Encoder, IoByteWriter, Node, NodeRef<'_>, W, server_to_domain_type(), W

### Community 157 - "StoreError"
Cohesion: 0.14
Nodes (12): PreKeyStore, Option, PreKeyRecordStructure, Send, SignedPreKeyRecordStructure, StoreError, Sync, Vec (+4 more)

### Community 158 - "prepare_group_stanza"
Cohesion: 0.12
Nodes (29): encrypt_group_message(), R, S, build_group_phash_set(), collect_stale_device_users(), create_sender_key_distribution_message_for_group(), prepare_group_retry_stanza(), prepare_group_stanza() (+21 more)

### Community 159 - "xml.rs"
Cohesion: 0.12
Nodes (16): DisplayableNode, DisplayableNodeRef, format_bytes_content(), format_node(), format_string_content(), get_printable_str(), Node, NodeRef<'a> (+8 more)

### Community 160 - "SessionState"
Cohesion: 0.11
Nodes (5): Chain, DecryptSnapshot, From, SessionState, SessionStructure

### Community 161 - "MessageContext"
Cohesion: 0.16
Nodes (13): BotHandle, default_http_client(), default_runtime(), default_transport_factory(), MessageContext, Arc, Client, ContextInfo (+5 more)

### Community 162 - "DeviceListRecord"
Cohesion: 0.13
Nodes (14): DeviceRegistryCache, DeviceTopology, Arc, AtomicU64, CompactString, Fn, IntoIterator, Item (+6 more)

### Community 163 - "wacore/src/usync.rs"
Cohesion: 0.18
Nodes (26): DeviceListResponse, LidQueryResponse, build_get_user_devices_query(), build_test_key_index_bytes(), build_usync_response(), parse_get_user_devices_response(), parse_get_user_devices_response_with_phash(), parse_lid_mappings_from_response() (+18 more)

### Community 164 - "wacore/src/prekeys.rs"
Cohesion: 0.17
Nodes (20): compute_key_bundle_digest(), create_mock_bundle(), decode_varint(), extract_prekey_public_key(), parse_keeps_companion_bundle_when_account_key_omitted_and_no_fallback(), parse_keeps_primary_bundle_ignoring_device_identity(), parse_one(), parse_skips_companion_bundle_with_invalid_device_identity() (+12 more)

### Community 165 - "UreqHttpClient"
Cohesion: 0.15
Nodes (17): Agent, build_agent(), execute_accepts_body_larger_than_ureq_default_limit(), parsed_content_length(), Default, Option, Receiver, Self (+9 more)

### Community 166 - "MessageInternalFields"
Cohesion: 0.10
Nodes (22): ContextInfoTag11InternalFields, ContextInfoTag15InternalFields, ContextInfoTag17InternalFields, ContextInfoTag1InternalFields, ContextInfoTag3InternalFields, ContextInfoTag4InternalFields, ContextInfoTag5InternalFields, ContextInfoTag6InternalFields (+14 more)

### Community 167 - "MessageInfo"
Cohesion: 0.13
Nodes (19): MessageServerId, Client, Arc, Message, Self, ChatMessageId, DeviceSentMeta, MessageCategory (+11 more)

### Community 168 - "WAPatchName"
Cohesion: 0.17
Nodes (11): Client, Arc, Self, SyncdMutation, Vec, Jid, server_jid(), MajorSyncTask (+3 more)

### Community 169 - "features/status.rs"
Cohesion: 0.14
Nodes (15): Client, Client, Into, Jid, Message, Option, Self, Vec (+7 more)

### Community 170 - "FlushScope"
Cohesion: 0.18
Nodes (18): close_rejects_new_tasks_until_reopened(), decrement_runs_when_future_is_aborted_mid_flight(), decrement_runs_when_future_is_dropped_before_first_poll(), DecrementOnDrop, flush_honors_timeout_and_logs(), flush_returns_after_tasks_complete(), flush_returns_immediately_when_idle(), FlushScope (+10 more)

### Community 171 - "Option"
Cohesion: 0.16
Nodes (20): checked_end(), ContextInfoInternalFields, extract_conversation_fields(), extract_own_pushname(), FieldIter<'a>, FieldWalker<'a>, HistorySyncError, HistorySyncResult (+12 more)

### Community 172 - "PrivateKey"
Cohesion: 0.17
Nodes (10): ConstantTimeEq, PrivateKey, CompressedEdwardsY, EdwardsPoint, From, R, Scalar, Self (+2 more)

### Community 173 - "features/contacts.rs"
Cohesion: 0.15
Nodes (18): ProfilePicture, Client, Contacts, Contacts<'a>, ensure_is_on_whatsapp_jids_supported(), forward_lid_pair(), is_on_whatsapp_accepts_pn_and_lid_jids(), is_on_whatsapp_rejects_unsupported_jid_type() (+10 more)

### Community 174 - "Client"
Cohesion: 0.10
Nodes (15): build_secret_message_edit(), Client, Arc, Fn, Into, Jid, Message, Node (+7 more)

### Community 175 - "Client"
Cohesion: 0.18
Nodes (7): Client, Arc, BytesMut, NodeRef, Option, Self, Semaphore

### Community 176 - "ui/tests.rs"
Cohesion: 0.15
Nodes (23): chat_flags_roll_back_on_failure(), deletes_roll_back_on_failure(), deleting_a_failed_bubble_is_local(), demo(), drafts_follow_the_chat(), edit_stashes_and_restores_the_composer(), env(), failed_send_is_marked_and_toasted() (+15 more)

### Community 177 - "UploadResponse"
Cohesion: 0.18
Nodes (21): audio_message(), AudioOptions, document_and_audio_set_type_specific_fields(), document_message(), DocumentOptions, image_maps_cdn_fields_and_defaults_mimetype(), image_message(), ImageOptions (+13 more)

### Community 178 - "resend_rate_limiter.rs"
Cohesion: 0.17
Nodes (17): buckets_are_per_chat(), chat(), concurrent_acquires_for_one_chat_do_not_exceed_burst(), disabled_limiter_allows_everything(), empty_bucket_refuses_then_refills_over_time(), idle_does_not_accumulate_beyond_burst(), ResendRateLimiter, Arc (+9 more)

### Community 179 - "wacore/src/store/commands.rs"
Cohesion: 0.23
Nodes (13): apply_command_to_device(), clear_server_cert_chain_drops_field(), dummy_chain(), increment_login_counter_bumps_and_saturates(), Device, set_server_cert_chain_populates_field(), set_then_clear_roundtrips(), test_history_sync_empty_salt_is_ignored() (+5 more)

### Community 180 - "CapturingMockTransport"
Cohesion: 0.11
Nodes (16): CapturingMockTransport, CapturingMockTransportFactory, MockTransport, MockTransportFactory, Arc, Bytes, Default, Mutex (+8 more)

### Community 181 - "PrivacyCategory"
Cohesion: 0.22
Nodes (12): DisallowedListAction, DisallowedListUpdate, DisallowedListUserEntry, PrivacyCategory, PrivacySetting, PrivacySettingsResponse, PrivacyValue, Jid (+4 more)

### Community 182 - "reporting_token.rs"
Cohesion: 0.15
Nodes (20): build_reporting_node(), decode_varint(), encode_varint(), encode_varint_to_buf(), extract_reporting_token_content(), generate_message_secret(), reporting_context_info(), ReportingField (+12 more)

### Community 183 - "stanza/receipt.rs"
Cohesion: 0.12
Nodes (15): collect_simple_message_ids(), downgrade_for_feature_incapable(), parse_participants(), participants_aggregated_by_message(), participants_aggregated_by_type(), participants_user_missing_t_yields_none_timestamp(), ReceiptUser, Jid (+7 more)

### Community 184 - "PublicKey"
Cohesion: 0.13
Nodes (10): Choice, KeyType, PublicKey, PublicKeyData, Display, Formatter, Ord, Ordering (+2 more)

### Community 185 - "notification/device.rs"
Cohesion: 0.19
Nodes (23): AccountSyncDevice, handle_account_sync_devices(), handle_account_sync_notification(), handle_devices_notification(), handle_digest_key(), handle_encrypt_notification(), handle_identity_change(), handle_local_identity_change() (+15 more)

### Community 186 - "Node"
Cohesion: 0.11
Nodes (26): AboutStatusInfo, Contacts, ContentAttributes, Creator, DefaultSubGroup, Description, Edges, Ephemeral (+18 more)

### Community 187 - "PersistenceManager"
Cohesion: 0.18
Nodes (10): PersistenceManager, Arc, AtomicBool, Device, Duration, Option, RwLock, Self (+2 more)

### Community 188 - "features/chat_actions.rs"
Cohesion: 0.11
Nodes (17): Collection, bool_str(), build_action_index(), ChatActions, collection_patch_name(), contact_action_index_and_collection(), contact_id_validation_accepts_only_bare_pn(), dispatch_chat_mutation() (+9 more)

### Community 189 - "Inner"
Cohesion: 0.17
Nodes (12): Inner, Arc, Cache, Duration, K, Option, PhantomData, Q (+4 more)

### Community 190 - "AudioPlayer"
Cohesion: 0.14
Nodes (19): Loaded, AudioPlayer, Cmd, decode_ogg_opus(), is_ogg_opus(), load(), lock(), PlayState (+11 more)

### Community 191 - "offline_resume.rs"
Cohesion: 0.19
Nodes (16): arm_publishes_first_batch_inflight(), batch_request_stanza_shape(), build_offline_batch_request(), first_arrival_cas_is_exclusive(), OfflineBatchCoordinator, on_offline_stanza_arrived(), reset_disarms_and_clears_inflight(), Arc (+8 more)

### Community 192 - "JidExt"
Cohesion: 0.15
Nodes (3): JidExt, JidRef<'a>, test_hosted_device_filtering_for_groups()

### Community 193 - "setup_session"
Cohesion: 0.13
Nodes (23): build_peer_stanza(), build_peer_stanza_with_options(), dm_retry_emits_enc_directly_under_message_with_recipient(), dm_retry_pkmsg_preflight_errors_when_account_missing(), dm_retry_pkmsg_targets_single_device(), dm_retry_pkmsg_with_account_has_device_identity(), dm_retry_preserves_edit_attribute(), group_retry_pkmsg_preflight_errors_when_account_missing() (+15 more)

### Community 194 - "features/events.rs"
Cohesion: 0.16
Nodes (16): EventMessage, EventResponseType, LocationMessage, build_event_message(), build_event_message_maps_fields(), Client, EventCreationParams, Events (+8 more)

### Community 195 - "CacheConfig"
Cohesion: 0.19
Nodes (14): CacheConfig, CacheEntryConfig, CacheStores, lid_pn_cache_default_is_effectively_unbounded(), Arc, Cache, Debug, Default (+6 more)

### Community 196 - "memory_soak.rs"
Cohesion: 0.23
Nodes (19): MemoryDiagnostics, Display, Formatter, analyze_growth(), env_or(), make_large_msg(), make_text_msg(), Client (+11 more)

### Community 197 - "labels.rs"
Cohesion: 0.23
Nodes (14): Client, dispatch_label_mutation(), label_edit_dispatches_update(), label_jid_dispatches_association(), label_jid_with_malformed_chat_is_claimed_but_not_dispatched(), label_methods_reject_empty_id(), Labels, non_label_kind_is_not_claimed() (+6 more)

### Community 198 - "MediaEncryptor"
Cohesion: 0.20
Nodes (10): EncryptedMediaInfo, media_encryptor_chunk_api(), media_encryptor_large_single_chunk(), media_encryptor_single_byte_chunks(), MediaEncryptor, Aes256, FnMut, Vec (+2 more)

### Community 199 - "Client"
Cohesion: 0.28
Nodes (9): Client, Arc, Jid, Message, MessageKey, Option, Self, StoreError (+1 more)

### Community 200 - "NodeRef"
Cohesion: 0.30
Nodes (18): check_usync_result_errors(), check_usync_result_errors_for(), parse_business_fields(), parse_contact_fields(), parse_lid_fields(), parse_picture_id_fields(), parse_subprotocol_error(), parse_user_common_fields() (+10 more)

### Community 201 - "NoiseSocket"
Cohesion: 0.20
Nodes (16): decrypt_frame_errors_on_counter_exhaustion(), NoiseSocket, Arc, AtomicU32, Bytes, BytesMut, Receiver, Self (+8 more)

### Community 202 - "PatchList"
Cohesion: 0.17
Nodes (20): CollectionSyncError, parse_collection_error(), parse_patch_list(), parse_patch_list_ref(), parse_patch_lists(), parse_patch_lists_ref(), parse_single_collection(), PatchList (+12 more)

### Community 203 - "poll.rs"
Cohesion: 0.24
Nodes (21): compute_option_hash(), decrypt_poll_vote(), decrypt_poll_vote_with_fallback(), decrypt_poll_vote_with_secret(), derive_vote_encryption_key(), empty_vote_roundtrip(), encrypt_poll_vote(), encrypt_poll_vote_with_secret() (+13 more)

### Community 204 - "util/jid.rs"
Cohesion: 0.14
Nodes (14): Chat, Section, display_name(), format_phone(), initials(), is_broadcast_list(), is_lid(), is_newsletter() (+6 more)

### Community 205 - "rust-backend/src/lib.rs"
Cohesion: 0.21
Nodes (7): ChatStateEvent, ChatstateHandler, Arc, Client, Jid, Option, Self

### Community 206 - "src/keepalive.rs"
Cohesion: 0.12
Nodes (7): classify_keepalive_error(), Client, is_benign_teardown(), KeepaliveResult, Arc, IqError, Self

### Community 207 - "Option"
Cohesion: 0.16
Nodes (21): bot_dm_reply_keeps_delivery_receipt(), bot_self_fanout_acked_via_sender_receipt(), confirmations_for(), decode_frame(), decrypt_failure_sends_retry_before_ack(), delivery_receipts_for(), find_message_ack_for(), find_receipt() (+13 more)

### Community 208 - "src/hash.rs"
Cohesion: 0.17
Nodes (17): create_mutation(), generate_patch_mac(), HashUpdateResult, F, HashMap, Option, Self, SyncdMutation (+9 more)

### Community 209 - "src/ib.rs"
Cohesion: 0.19
Nodes (12): IbContent, IbStanza, Into, Node, NodeRef, Self, test_ib_stanza_into_node(), test_unified_session_calculate_id() (+4 more)

### Community 210 - "NodeRef"
Cohesion: 0.10
Nodes (8): GetGroupInviteInfoIq, GroupCreateIq, GroupParticipatingIq, JoinLinkedGroupIq, parse_group_id(), parse_join_group_response(), QueryLinkedGroupIq, NodeRef

### Community 211 - "src/message_edit.rs"
Cohesion: 0.26
Nodes (19): decrypt_message_edit(), decrypt_message_edit_with_fallback(), decrypt_secret_encrypted(), decrypt_secret_encrypted_with_fallback(), encrypt_decrypt_roundtrip_text(), encrypt_decrypt_roundtrip_with_lid_jids(), encrypt_message_edit(), fallback_recovers_on_alternate_jid_form() (+11 more)

### Community 212 - "secret_enc_addon.rs"
Cohesion: 0.22
Nodes (16): MessageEditContext<'a>, aad_mismatch_under_same_key_fails_decrypt(), AadMode, AddonContext, build_aad(), ctx(), decrypt_addon(), decrypt_invalid_iv_size() (+8 more)

### Community 213 - "PairCodeUtils"
Cohesion: 0.12
Nodes (14): PairCodeUtils, Duration, test_code_validity_duration(), test_crockford_encoding_boundary_values(), test_decrypt_invalid_length(), test_decrypt_with_wrong_code_produces_garbage(), test_derive_key_deterministic(), test_derive_key_with_different_salts() (+6 more)

### Community 214 - "generate_reporting_token_content"
Cohesion: 0.14
Nodes (20): extract_message_secret(), generate_reporting_token_content(), prepare_message_with_context(), reporting_token_reuses_existing_message_secret(), Message, should_include_reporting_token(), test_context_info_filtering_only_extracts_whitelisted(), test_excluded_message_types() (+12 more)

### Community 215 - "Persist"
Cohesion: 0.16
Nodes (8): Persist, HashMap, Option, Path, PathBuf, Self, T, Vec

### Community 216 - "BotBuilder<B, T, H, R>"
Cohesion: 0.25
Nodes (4): Eh, BotBuilder<B, T, H, R>, F, Self

### Community 217 - "IqError"
Cohesion: 0.18
Nodes (12): IqSendFuture, Client, converts_unexpected_response_type(), IqError, Arc, Box, Duration, From (+4 more)

### Community 218 - "ClientProfile"
Cohesion: 0.12
Nodes (18): Platform, apply_command_to_device(), Device, android_profile_omits_web_info_and_carries_os_version(), ClientProfile, ios_profile_marks_apple_manufacturer(), native_profiles_all_drop_web_info(), Default (+10 more)

### Community 219 - "Client"
Cohesion: 0.21
Nodes (5): Client, IqError, Jid, Option, SetPrivacySettingResponse

### Community 220 - "aes_256_cbc_encrypt_into"
Cohesion: 0.33
Nodes (13): aes_256_cbc_decrypt_into(), aes_256_cbc_encrypt_into(), DecryptionError, EncryptionError, Vec, test_aes_256_cbc_nist_vector(), test_aes_cbc_decrypt_bad_ciphertext_length(), test_aes_cbc_decrypt_wrong_key() (+5 more)

### Community 221 - "controller.rs"
Cohesion: 0.16
Nodes (18): Rgba8Pixel, SharedPixelBuffer, GifState, Preview, Rec, HashMap, Option, PathBuf (+10 more)

### Community 222 - "RetryReason"
Cohesion: 0.20
Nodes (11): Level, decrypt_fail_log_level(), Client, Arc, Jid, Option, Self, RetryReason (+3 more)

### Community 224 - "ClientError"
Cohesion: 0.20
Nodes (9): ClientError, Chatstate, Chatstate<'a>, ChatStateType, Client, Client, Jid, Node (+1 more)

### Community 225 - "reporting_token_benchmark.rs"
Cohesion: 0.22
Nodes (17): bench_content_extraction_extended(), bench_content_extraction_simple(), bench_full_token_generation_extended(), bench_full_token_generation_simple(), bench_message_encoding_extended(), bench_message_encoding_simple(), create_extended_message(), create_simple_message() (+9 more)

### Community 226 - "curve.rs"
Cohesion: 0.26
Nodes (15): EdwardsCacheData, KeyPair, prepared_verifier_matches_plain_verify(), rng(), CompressedEdwardsY, CryptoRng, Scalar, test_cloned_key_signatures() (+7 more)

### Community 227 - "Message"
Cohesion: 0.20
Nodes (6): build_quote_context(), MessageBuilderExt, ContextInfo, Into, Message, wa::Message

### Community 228 - "wire_enum_serde_test.rs"
Cohesion: 0.12
Nodes (7): DeviceNotificationType, DeviceListUpdateType, From, addressing_mode_matches_wire(), assert_roundtrip(), device_notification_type_uses_wire_strings(), T

### Community 229 - "GroupInfoResponse"
Cohesion: 0.08
Nodes (18): BatchGroupInfoResult, generate_description_id(), group_query_iq_with_phash_emits_attr(), GroupInfoOutcome, GroupInfoResponse, GroupParticipantResponse, GroupParticipatingResponse, GroupProfilePicture (+10 more)

### Community 230 - "message_utils_benchmark.rs"
Cohesion: 0.21
Nodes (16): bench_decode_plaintext(), bench_dm_send_encode_work(), bench_encode_and_pad(), bench_parse_message_info(), bench_participant_list_hash_1600(), bench_unpad_message_ref(), dm_shape(), msg_info_node() (+8 more)

### Community 231 - "PrivateKey"
Cohesion: 0.15
Nodes (9): PrivateKey, PrivateKeyData, Deserialize, Eq, OnceLock, PartialEq, R, S (+1 more)

### Community 232 - "SendResult"
Cohesion: 0.19
Nodes (11): both_call_styles_compile(), pin_message_rejects_newsletter(), PinDuration, Into, Message, MessageKey, send_message_to_status_reaction_rejects_non_user_participant(), send_message_to_status_reaction_without_participant_errors() (+3 more)

### Community 233 - "VoiceRecorder"
Cohesion: 0.12
Nodes (15): Captured, encode_ogg_opus(), opus_roundtrip(), resample(), Arc, AtomicBool, JoinHandle, Mutex (+7 more)

### Community 234 - "bot.rs"
Cohesion: 0.15
Nodes (18): BotBuilder, create_test_sqlite_backend_for_device(), MissingBackend, MissingHttpClient, MissingRuntime, MissingTransport, react_info(), react_target_key_carries_group_participant() (+10 more)

### Community 235 - "Vec"
Cohesion: 0.21
Nodes (13): build_user_nodes(), DeviceListSpec, IsOnWhatsAppQueryType, IsOnWhatsAppSpec, IsOnWhatsAppUser, pn_user(), CompactString, HashMap (+5 more)

### Community 236 - "ChainKey"
Cohesion: 0.12
Nodes (10): InitialPQRKey, derive_keys(), derive_keys_with_label(), ChainKey, RootKey, Display, Formatter, PrivateKey (+2 more)

### Community 237 - "SenderKeyDeviceCache"
Cohesion: 0.18
Nodes (8): Arc, Cache, F, HashMap, Option, Self, SenderKeyDeviceCache, SenderKeyDeviceMap

### Community 238 - "SecretBytes"
Cohesion: 0.16
Nodes (8): HistorySyncStream<'a>, Debug, Deref, Eq, Formatter, PartialEq, Self, SecretBytes

### Community 239 - "iq/business.rs"
Cohesion: 0.21
Nodes (13): BusinessCategory, BusinessHourMode, BusinessHours, BusinessHoursConfig, BusinessProfile, BusinessProfileSpec, DayOfWeek, node_text() (+5 more)

### Community 240 - "MessageExt"
Cohesion: 0.14
Nodes (3): MessageExt, Box, Option

### Community 241 - "ResolvedGroupDevices"
Cohesion: 0.23
Nodes (11): jid(), phash_memo_matches_direct_compute_and_pins_sender(), ResolvedGroupDevices, CompactString, Debug, Formatter, Jid, OnceLock (+3 more)

### Community 242 - "assemble_status_participants"
Cohesion: 0.15
Nodes (16): assemble_status_participants(), ensure_status_participants(), I, Jid, Message, Node, Vec, status_carries_privacy_meta() (+8 more)

### Community 243 - "waproto/src/lib.rs"
Cohesion: 0.21
Nodes (16): history_sync_decode(), message_context_info_encode_into(), message_context_info_encoded_len(), message_context_info_merge(), message_context_info_to_vec(), message_decode(), message_encode_into(), message_encoded_len() (+8 more)

### Community 244 - "Downloadable"
Cohesion: 0.19
Nodes (5): Downloadable, MockDownloadable, Option, Send, Sync

### Community 245 - "Bot"
Cohesion: 0.13
Nodes (14): EventHandlerCallback, Bot, BotBuilder<Provided, Provided, Provided, Provided>, BotBuilderError, CallbackBusAdapter, combined_interest(), combined_interest_is_the_union_of_handler_interests(), RegisteredHandler (+6 more)

### Community 246 - ".issue_tokens"
Cohesion: 0.17
Nodes (9): Client, Client, IqError, Jid, Option, Self, Vec, TcToken (+1 more)

### Community 247 - "DevicePropsOverride"
Cohesion: 0.19
Nodes (9): AppVersion, ClientPayload, HistorySyncConfig, build_base_client_payload(), default_history_sync_config(), DevicePropsOverride, Into, PlatformType (+1 more)

### Community 248 - "test_utils.rs"
Cohesion: 0.30
Nodes (13): create_test_backend(), create_test_client(), create_test_client_with_config(), create_test_client_with_failing_http(), create_test_client_with_http(), create_test_client_with_name(), node_to_owned_ref(), Arc (+5 more)

### Community 249 - ".get_user_devices"
Cohesion: 0.23
Nodes (10): Client, process_response_preserves_omitted_users(), process_response_skips_empty_device_list(), process_response_warms_lid_pn_cache_synchronously(), Jid, Vec, test_device_registry_db_fallback(), test_device_registry_hit_for_lid_jid() (+2 more)

### Community 250 - "CurveError"
Cohesion: 0.35
Nodes (4): CurveError, D, Ok, Self

### Community 251 - "record_helpers.rs"
Cohesion: 0.27
Nodes (13): new_signed_pre_key_record(), prekey_record_to_structure(), prekey_structure_to_record(), Box, DateTime, PreKeyRecordStructure, SignalProtocolError, SignedPreKeyRecordStructure (+5 more)

### Community 252 - "EnforcementExtraData"
Cohesion: 0.20
Nodes (14): AppealExtraData, BaseEnforcementData, BaseEnforcementData2, ContentData, EnforcementExtraData, EnforcementExtraData2, EnforcementExtraData3, EnforcementPolicyInformation (+6 more)

### Community 253 - "zip.rs"
Cohesion: 0.33
Nodes (11): central_directory_offsets_valid(), crc32(), Entry, put_u16(), put_u32(), put_zero_u16s(), Self, Vec (+3 more)

### Community 254 - "Server"
Cohesion: 0.11
Nodes (9): DeviceKey, push_jid_to_compact(), push_jid_to_string(), Deserialize, PartialEq, Serialize, TryFrom, Server (+1 more)

### Community 255 - ".update_local_signal_session"
Cohesion: 0.19
Nodes (16): build_retry_receipt_with_registration(), build_retry_receipt_without_keys(), concurrent_same_peer_recreate_check_is_serialized(), dm_retry_info(), ensure_e2e_sessions_resolved_is_noop_when_session_exists(), extract_registration_id_from_node(), get_bytes_content(), Node (+8 more)

### Community 256 - "state/session.rs"
Cohesion: 0.38
Nodes (12): create_test_message_key_generator(), create_test_session_state(), rng(), CryptoRng, test_message_keys_eviction_at_max(), test_message_keys_lookup_by_counter_not_order(), test_promote_matching_session_already_current(), test_promote_matching_session_finds_correct_session() (+4 more)

### Community 257 - "prep.rs"
Cohesion: 0.26
Nodes (15): compress_image(), compresses_and_thumbnails(), decode(), encode_jpeg(), is_gif(), is_image_path(), mime_for_path(), DynamicImage (+7 more)

### Community 258 - "DownloadParams"
Cohesion: 0.24
Nodes (3): DownloadParams, PlaintextDownloadable, Option

### Community 259 - ".send_message"
Cohesion: 0.22
Nodes (9): Client, Comments, Comments<'a>, Client, Into, Jid, Message, MessageKey (+1 more)

### Community 260 - "Jid"
Cohesion: 0.23
Nodes (12): Client, extract_registration_id_from_node_ref(), get_bytes_content_ref(), is_fbid_bot_retry_jid(), maybe_resolve_retry_chat_info(), resolve_retry_chat_info_peer_device_without_recipient(), retry_key_bundle_requires_one_time_prekey_except_fbid_bot(), RetryChatInfo (+4 more)

### Community 261 - "EventRecorder"
Cohesion: 0.24
Nodes (8): acquire_permit_old_behavior(), acquire_permit_with_reacquire(), EventRecorder, AtomicU64, Mutex, Semaphore, test_old_behavior_drops_tasks_on_generation_swap(), test_semaphore_generation_swap_does_not_drop_tasks()

### Community 262 - "enc_handler.rs"
Cohesion: 0.25
Nodes (11): EncHandler, MockEncHandler, Arc, Client, Mutex, Node, Self, Vec (+3 more)

### Community 263 - "GatewayImpl"
Cohesion: 0.22
Nodes (9): GatewayImpl, Arc, Client, Option, RwLock, Send, Sync, Vec (+1 more)

### Community 264 - "ExpandedAppStateKeys"
Cohesion: 0.09
Nodes (35): bench_decode_record(), bench_expand_app_state_keys(), bench_lthash_subtract_then_add_812(), bench_process_patch_50_validated(), PatchFixture, Arc, Bencher, HashMap (+27 more)

### Community 265 - "CameraSession"
Cohesion: 0.16
Nodes (11): RgbImage, CameraSession, Arc, AtomicBool, Drop, Fn, Mutex, Option (+3 more)

### Community 266 - "Instant"
Cohesion: 0.26
Nodes (7): Add, Instant, now_nanos(), Duration, Self, StdMonotonicProvider, Sub

### Community 267 - "wacore"
Cohesion: 0.34
Nodes (14): bench-integration, e2e-tests, wacore, wacore-appstate, wacore-binary, wacore-derive, wacore-libsignal, wacore-noise (+6 more)

### Community 268 - "Client"
Cohesion: 0.22
Nodes (7): Client, Arc, CompactString, HashMap, Jid, Option, Vec

### Community 269 - "Profile<'a>"
Cohesion: 0.20
Nodes (6): Client, Profile, Profile<'a>, Client, Self, Vec

### Community 270 - ".sleep"
Cohesion: 0.34
Nodes (9): Box, Duration, FnOnce, Future, Option, Output, Pin, Send (+1 more)

### Community 271 - "token.rs"
Cohesion: 0.26
Nodes (10): get_double_token(), get_single_token(), index_of_token(), lookup_matches_reference_under_byte_mutation(), Option, test_almost_matching_strings(), test_double_byte_token_roundtrip(), test_single_byte_token_roundtrip() (+2 more)

### Community 272 - "SignalProtocolError"
Cohesion: 0.16
Nodes (10): DummyBackendError, Box, E, FnOnce, From, Self, Send, Sync (+2 more)

### Community 273 - "edge_routing.rs"
Cohesion: 0.25
Nodes (12): build_edge_routing_preintro(), build_handshake_header(), EdgeRoutingError, Option, Vec, test_build_edge_routing_preintro_basic(), test_build_edge_routing_preintro_empty(), test_build_edge_routing_preintro_large_length() (+4 more)

### Community 274 - "build_reaction_message"
Cohesion: 0.21
Nodes (14): build_keep_in_chat_message(), build_keep_in_chat_message_keep_for_all(), build_keep_in_chat_message_undo(), build_reaction_edit_attr_classification(), build_reaction_empty_emoji_is_unreact_form(), build_reaction_message(), build_reaction_populates_key_text_and_timestamp(), build_reaction_preserves_participant_for_group_target() (+6 more)

### Community 275 - "ConversationExt"
Cohesion: 0.21
Nodes (4): ConversationExt, Jid, Vec, wa::Conversation

### Community 276 - "protocol/keepalive.rs"
Cohesion: 0.18
Nodes (5): is_dead_socket(), ms_since(), ms_since_recent(), ms_since_stale(), Option

### Community 277 - "Device"
Cohesion: 0.15
Nodes (17): deserialize(), Device, from_bytes(), AdvSignedDeviceIdentity, Arc, D, DecodeError, Default (+9 more)

### Community 278 - "notification.rs"
Cohesion: 0.22
Nodes (11): notification_timestamp(), notification_timestamp_missing_t(), notification_timestamp_with_valid_t(), parse_disappearing_mode(), parse_disappearing_mode_disabled(), parse_disappearing_mode_missing_duration_defaults_to_zero(), parse_disappearing_mode_valid(), DateTime (+3 more)

### Community 279 - "create_record_with_previous_sessions"
Cohesion: 0.22
Nodes (9): create_record_with_previous_sessions(), ExactSizeIterator, Item, test_session_record_serialization_preserves_previous_sessions(), test_take_out_of_bounds(), test_take_restore_at_various_indices(), test_take_restore_maintains_count(), test_take_restore_preserves_order() (+1 more)

### Community 280 - ".cast"
Cohesion: 0.40
Nodes (4): B2, H2, R2, T2

### Community 281 - "ImageCache"
Cohesion: 0.23
Nodes (11): decode_b64(), decode_file(), ImageCache, DynamicImage, HashMap, Option, Path, Pixels (+3 more)

### Community 282 - "PreparedVerifyingKey"
Cohesion: 0.21
Nodes (7): PreparedEdwards, PreparedVerifyingKey, Arc, Debug, EdwardsPoint, From, Option

### Community 283 - "Client"
Cohesion: 0.21
Nodes (6): Client, Arc, Device, Mutex, Option, RwLock

### Community 284 - "Client"
Cohesion: 0.33
Nodes (4): Client, Jid, Message, Option

### Community 285 - "notification/profile.rs"
Cohesion: 0.37
Nodes (12): handle_contacts_notification(), handle_picture_notification(), handle_status_notification(), learn_contact_modify_mappings(), notification_timestamp(), Arc, Client, DateTime (+4 more)

### Community 286 - "rust-backend/src/pair.rs"
Cohesion: 0.32
Nodes (12): handle_iq(), handle_pair_success(), make_qr_data(), make_qr_data_with_client_type(), pair_with_qr_code(), Arc, Client, Device (+4 more)

### Community 287 - "InfoQuery"
Cohesion: 0.07
Nodes (22): iq_structure(), RemoveCompanionDeviceSpec, Jid, Self, AcceptGroupInviteIq, GroupSubject, SetGroupMembershipApprovalIq, SetGroupSubjectIq (+14 more)

### Community 288 - ".reader_from"
Cohesion: 0.24
Nodes (7): bytes::Bytes, Box, Read, Send, Sync, std::sync::Arc<[u8]>, UploadSource

### Community 289 - "appstate_external_mutations_test.rs"
Cohesion: 0.26
Nodes (10): create_value_blob(), make_mutation(), Option, SyncdMutation, SyncdOperation, Vec, test_external_mutations_decode_from_syncd_mutations(), test_has_missing_remove_flag_not_set_on_set_without_previous_value() (+2 more)

### Community 290 - "Phase 2 Frontend"
Cohesion: 0.22
Nodes (14): Phase 2 Frontend, Constraint: low RAM webview frontend, Event -> store data flow (stores/events.ts), IndexedDB persistence (lib/persist.ts), LID <-> phone number identity unification, Message lifecycle (edits, deletes, receipts, reactions), Part C: sending media (planned), UI zoom (lib/zoom.ts) (+6 more)

### Community 291 - "Device"
Cohesion: 0.21
Nodes (6): DerefMut, Device, Arc, Deref, Self, Target

### Community 292 - "ProtocolNode"
Cohesion: 0.21
Nodes (8): parse_string_enum(), ParseStringEnum, ProtocolNode, Node, NodeRef, Self, T, Sized

### Community 293 - ".pair_with_code"
Cohesion: 0.21
Nodes (8): Client, handle_pair_code_notification(), PairError, Arc, IqError, NodeRef, Self, PairCodeError

### Community 294 - "generate_reporting_token"
Cohesion: 0.20
Nodes (12): bench_token_calculation(), calculate_reporting_token(), generate_reporting_token(), Jid, test_calculate_reporting_token(), test_generate_reporting_token_full(), test_generate_reporting_token_with_existing_secret(), test_golden_full_token_generation() (+4 more)

### Community 295 - "BenchRuntime"
Cohesion: 0.39
Nodes (8): BenchRuntime, Box, Duration, FnOnce, Future, Output, Pin, Send

### Community 296 - "Option"
Cohesion: 0.18
Nodes (9): DeviceListUpdate, DeviceNotificationInfo, IdentityChange, NewsletterLiveUpdate, NewsletterLiveUpdateMessage, NewsletterLiveUpdateReaction, Message, Option (+1 more)

### Community 297 - "Self"
Cohesion: 0.14
Nodes (7): Encoder<'a, SliceByteWriter<'a>>, Encoder<'static, VecByteWriter<'v>>, Option, Self, Vec, SliceByteWriter<'a>, VecByteWriter<'a>

### Community 298 - "PairUtils"
Cohesion: 0.27
Nodes (6): PairUtils, Jid, Node, NodeRef, Option, Vec

### Community 299 - ".spawn"
Cohesion: 0.39
Nodes (8): pin(), Box, Duration, FnOnce, Future, Output, Send, TokioTestRuntime

### Community 300 - "CacheStore"
Cohesion: 0.20
Nodes (6): CacheStore, Duration, Option, Send, Sync, Vec

### Community 301 - "ApiError"
Cohesion: 0.15
Nodes (8): ApiError, Display, Self, EmojiCategory, Option, Vec, search(), shortcode_token()

### Community 302 - "measure_binary_size.py"
Cohesion: 0.44
Nodes (10): build_env(), count_lockfile_crates(), main(), measure_sections(), measure_stripped_size(), Path, Measure binary-size metrics for the size-tracking CI job. Builds the `whatsapp-…, run() (+2 more)

### Community 303 - "PendingDeviceSync"
Cohesion: 0.25
Nodes (6): PendingDeviceSync, HashSet, Jid, Mutex, Self, Vec

### Community 304 - "scan_message_level"
Cohesion: 0.25
Nodes (11): carrier_slot(), fast_extract(), FastExtract, FastRecord, MsgLevel, From, scan_context_info(), scan_message_level() (+3 more)

### Community 305 - "prepare_peer_stanza"
Cohesion: 0.33
Nodes (10): prepare_peer_stanza(), prepare_peer_stanza_with_options(), AdvSignedDeviceIdentity, I, Jid, Message, Node, Option (+2 more)

### Community 306 - "wacore/src/store/error.rs"
Cohesion: 0.25
Nodes (9): database_preserves_typed_source_via_downcast(), DummyBackendError, is_busy_or_locked_is_case_insensitive(), is_busy_or_locked_negative(), is_busy_or_locked_walks_chain(), Box, Send, Sync (+1 more)

### Community 307 - "src/reaction.rs"
Cohesion: 0.38
Nodes (8): ReactionMessage, decrypt_reaction_with_secret(), empty_text_removal_roundtrips(), encrypt_reaction_with_secret(), reaction_addon_ctx(), roundtrip_keeps_text_and_timestamp_only(), Vec, wrong_reactor_fails_decrypt()

### Community 308 - "EncPayload"
Cohesion: 0.40
Nodes (7): EncPayload, Bytes, NodeRef, Option, Self, msmsg_payload_from_bytes(), skmsg_payload_from_bytes()

### Community 309 - "comment.rs"
Cohesion: 0.42
Nodes (9): body(), comment_addon_ctx(), decrypt_comment_with_secret(), encrypt_comment_with_secret(), invalid_secret_size_rejected(), roundtrip_extended_text_body(), Message, Vec (+1 more)

### Community 310 - ".prepare_for_quote"
Cohesion: 0.20
Nodes (9): test_prepare_for_quote_breaks_quote_chain(), test_prepare_for_quote_device_sent_message(), test_prepare_for_quote_edited_message(), test_prepare_for_quote_nested_wrappers(), test_prepare_for_quote_preserves_bot_quote_chain(), test_prepare_for_quote_preserves_bot_server_quote_chain(), test_prepare_for_quote_preserves_media_fields(), test_prepare_for_quote_strips_mentions_preserves_content() (+1 more)

### Community 311 - ".poll"
Cohesion: 0.20
Nodes (8): Context, Drop, Future, Output, Pin, Poll, Self, Spawned<T>

### Community 312 - "LocalChatSettings"
Cohesion: 0.22
Nodes (8): Details, LocalChatSettings, Box, DateTime, Option, Utc, VerifiedNameCertificate, VerifiedName

### Community 313 - "waproto/build.rs"
Cohesion: 0.36
Nodes (8): FileDescriptorSet, ensure_proto_descriptor_hash(), ExpectedHashes, generate_tags(), main(), read_expected_hashes(), Path, sha256_hex()

### Community 314 - "binary_size_report.py"
Cohesion: 0.53
Nodes (8): fmt_delta(), fmt_value(), human_bytes(), load_metrics(), main(), Path, Build the binary-size PR report and evaluate the regression gate. Compares the…, top_movers()

### Community 315 - "notification/groups.rs"
Cohesion: 0.56
Nodes (8): handle_disappearing_mode_notification(), handle_group_notification(), handle_mex_notification(), handle_newsletter_notification(), handle_server_sync_notification(), Arc, Client, NodeRef

### Community 316 - ".handle_sender_key_distribution_message"
Cohesion: 0.28
Nodes (6): Client, AppStateSyncKeyShare, Arc, Jid, NodeRef, Self

### Community 317 - "history_sync_benchmark.rs"
Cohesion: 0.33
Nodes (7): bench_history_sync_stream_drain(), bench_process_history_sync(), build_realistic_history_sync(), pseudo_text(), Bencher, Vec, setup_history_sync_blob()

### Community 318 - "Settings"
Cohesion: 0.21
Nodes (7): Default, Option, PathBuf, Self, Vec, Settings, zoom_and_recents()

### Community 319 - "rust-backend/src/main.rs"
Cohesion: 0.43
Nodes (7): build_media_pong(), handle_send_command(), handle_text_ping(), main(), parse_arg(), Message, Option

### Community 320 - "required_attr"
Cohesion: 0.14
Nodes (12): LinkSubgroupsIq, UnlinkSubgroupsIq, collect_children(), extract_content_bytes(), extract_content_uint(), optional_child(), required_attr(), Cow (+4 more)

### Community 321 - "derive_reporting_token_key"
Cohesion: 0.29
Nodes (8): bench_key_derivation(), build_hkdf_info(), derive_key_matches_plain_hkdf_extract(), derive_reporting_token_key(), Vec, test_derive_reporting_token_key(), test_golden_hkdf_key_derivation(), test_hkdf_info_construction()

### Community 322 - "jid_benchmark.rs"
Cohesion: 0.36
Nodes (6): bench_jid_parse(), bench_jid_push_ad_to(), bench_jid_to_non_ad_string(), bench_jid_to_string(), Bencher, Jid

### Community 323 - "AttrsRef<'static>"
Cohesion: 0.32
Nodes (4): AttrsRef<'static>, F, Output, Yokeable

### Community 324 - "iq/keepalive.rs"
Cohesion: 0.31
Nodes (8): KeepaliveSpec, Duration, NodeRef, Option, Self, test_keepalive_spec_build_iq(), test_keepalive_spec_parse_response(), test_keepalive_spec_with_timeout()

### Community 325 - "utils.rs"
Cohesion: 0.50
Nodes (7): constant_time_cmp(), ct_is_eq(), ct_is_lt(), ct_is_zero(), ct_select(), expand_top_bit(), Ordering

### Community 326 - ".prepare_for_forward"
Cohesion: 0.25
Nodes (7): prepare_for_forward_bumps_score_when_source_already_forwarded(), prepare_for_forward_clears_bot_quote_chain(), prepare_for_forward_jumps_to_sentinel_at_threshold(), prepare_for_forward_marks_fresh_conversation(), prepare_for_forward_marks_ptv_message(), prepare_for_forward_strips_quote_chain(), strip_nested_context_info()

### Community 327 - "types/presence.rs"
Cohesion: 0.29
Nodes (4): ChatPresenceUpdate, ChatPresence, ChatPresenceMedia, Presence

### Community 328 - "klipy.rs"
Cohesion: 0.29
Nodes (12): api_key(), download(), encode_query(), fetch(), Gif, normalize(), normalizes_items(), Page (+4 more)

### Community 329 - "rust-backend/src/version.rs"
Cohesion: 0.13
Nodes (7): fetch_latest_app_version(), resolve_and_update_version(), Arc, Option, PersistenceManager, parse_sw_js(), Option

### Community 330 - "PairCodeState"
Cohesion: 0.18
Nodes (8): PairCodeState, Box, Debug, Formatter, KeyPair, NodeRef, Option, Vec

### Community 331 - "attach.rs"
Cohesion: 0.21
Nodes (7): ctrl(), Api, Controller, Future, Option, Output, T

### Community 333 - ".send_reaction"
Cohesion: 0.52
Nodes (4): Client, Into, Jid, MessageKey

### Community 334 - "bench_collect_unique_index_macs"
Cohesion: 0.33
Nodes (5): bench_collect_unique_index_macs(), Bencher, SyncdMutation, Vec, setup_mutations()

### Community 335 - "BenchRng"
Cohesion: 0.43
Nodes (3): BenchRng, TryCryptoRng, TryRng

### Community 336 - "CountingAlloc"
Cohesion: 0.33
Nodes (3): CountingAlloc, GlobalAlloc, Layout

### Community 337 - "noise_benchmark.rs"
Cohesion: 0.43
Nodes (5): bench_frame_decrypt_in_place(), bench_frame_encrypt_in_place(), frame(), Bencher, Vec

### Community 338 - "CoreClient"
Cohesion: 0.48
Nodes (3): CoreClient, Device, Self

### Community 339 - "Architecture & Process"
Cohesion: 0.33
Nodes (9): Architecture & Process, src-tauri backend glue, Tauri Commands (invoke), Tauri Events (emit/listen), Tauri monolith + native IPC decision, Three layers: protocol core, src-tauri glue, Svelte frontend, lib/ipc.ts IPC wrapper, Svelte 5 + Vite + TypeScript stack (+1 more)

### Community 340 - "privacy_business.rs"
Cohesion: 0.73
Nodes (5): handle_business_notification(), handle_privacy_token_notification(), Arc, Client, NodeRef

### Community 342 - "Phase 1 API surface & IPC mapping"
Cohesion: 0.36
Nodes (9): Phased plan (1 backend, 2A/2B frontend, 3 testing, 4 packaging), Phase 1 API surface & IPC mapping, ApiError {code,message}, AppState (BotHandle, Arc<Client>), Commands: auth, messaging, chats, groups, presence, profile, Event bridge (ChannelEventHandler -> app.emit), Events wa://<area>/<thing>, DTOs crossing IPC (+1 more)

### Community 343 - "HandshakeUtils"
Cohesion: 0.31
Nodes (3): HandshakeMessage, HandshakeUtils, ServerHello

### Community 344 - "Schema"
Cohesion: 0.36
Nodes (6): by_name(), Collection, IndexPart, Option, Schema, Scope

### Community 345 - "CountingAlloc"
Cohesion: 0.50
Nodes (3): CountingAlloc, GlobalAlloc, Layout

### Community 346 - "main"
Cohesion: 0.40
Nodes (4): main(), Box, Vec, Tokens

### Community 347 - "NoiseCipher"
Cohesion: 0.28
Nodes (5): NoiseCipher, B, Box, Self, test_noise_cipher_in_place_roundtrip()

### Community 348 - "build_zero_signed_chain"
Cohesion: 0.60
Nodes (4): build_zero_signed_chain(), Vec, verify_server_cert_rejects_fixture_with_zero_signed_certs(), verify_server_cert_rejects_when_leaf_key_does_not_match_static()

### Community 349 - ".into_base_message"
Cohesion: 0.40
Nodes (4): test_into_base_message_empty_dsm(), test_into_base_message_passthrough_plain(), test_into_base_message_unwraps_device_sent_reaction(), test_into_base_message_unwraps_nested_dsm_ephemeral()

### Community 350 - "handlers/ib.rs"
Cohesion: 0.43
Nodes (5): handle_ib_impl(), IbHandler, Arc, Client, NodeRef

### Community 351 - "Receipt"
Cohesion: 0.25
Nodes (7): build_retry_processing_key(), handle_retry_receipt_drops_throttled_group_resend(), retry_processing_key_per_participant(), Arc, Self, Receipt, MessageId

### Community 352 - "handlers/presence.rs"
Cohesion: 0.47
Nodes (3): PresenceHandler, Arc, Client

### Community 353 - "DisconnectReason"
Cohesion: 0.18
Nodes (6): mock_admin_scan_qr_url(), Option, DisconnectReason, Display, Formatter, Option

### Community 355 - "build_ephemeral_setting_message"
Cohesion: 0.60
Nodes (4): build_ephemeral_setting_message(), ephemeral_setting_disable_uses_zero_duration(), ephemeral_setting_message_shape(), Message

### Community 356 - "abprops.rs"
Cohesion: 0.83
Nodes (3): AbDefault, AbProp, AbPropType

### Community 357 - "MessageStanza"
Cohesion: 0.67
Nodes (3): MessageStanza, Jid, Option

### Community 360 - "Recorder"
Cohesion: 0.50
Nodes (3): Recorder, Arc, Mutex

### Community 361 - "ConnectionOptions"
Cohesion: 0.67
Nodes (3): CustomizeConnection, ConnectionOptions, SqliteConnection

### Community 363 - "seed_peer_send_state"
Cohesion: 0.29
Nodes (8): dm_to_lid_mapped_peer_addresses_outer_to_by_lid(), pdo_request_message(), peer_pdo_send_path_stamps_history_sync_options(), peer_test_account_proto(), AdvSignedDeviceIdentity, PeerDataOperationRequestType, seed_peer_send_state(), stanza_type_override_sets_wire_type_attr()

### Community 364 - "ReceiptType"
Cohesion: 0.39
Nodes (4): ReceiptType, From, Option, Self

### Community 393 - "paths.rs"
Cohesion: 0.54
Nodes (7): app_data_dir(), data_dir(), download_dir(), media_dir(), Option, PathBuf, ui_cache_dir()

### Community 394 - "noise/src/handshake.rs"
Cohesion: 0.18
Nodes (13): EncodeError, HandshakeError, ik_with_wrong_server_static_fails_at_decrypt(), IkFallbackInputs, IkHandshakeOutcome, IkServerHelloOutcome, Box, DecodeError (+5 more)

### Community 395 - "Tauri desktop app (whatsvelte-rust)"
Cohesion: 0.38
Nodes (7): Media & content types (asset protocol), Release Tauri App workflow, OS build matrix (ubuntu, macos, windows x64/arm64), Version tag v* trigger, tauri-action cross-platform build, Phase 4: Packaging & CI/CD, Tauri desktop app (whatsvelte-rust)

### Community 396 - "Tauri + Svelte → Slint migration plan"
Cohesion: 0.29
Nodes (6): Feature parity checklist, Mapping, Phases (each lands as its own commit), Target layout, Tauri + Svelte → Slint migration plan, Verification notes

### Community 398 - "slint-client/src/main.rs"
Cohesion: 0.52
Nodes (6): debug_snapshot(), install_window_hooks(), main(), AppWindow, Arc, Controller

### Community 399 - "Protocol Architecture doc"
Cohesion: 0.53
Nodes (6): Step-by-step feature implementation flow, Protocol Architecture doc, Derive macros (ProtocolNode, EmptyNode, StringEnum), IqSpec trait, Noise handshake patterns & invalidation, ProtocolNode trait

### Community 400 - "handlers/receipt.rs"
Cohesion: 0.47
Nodes (3): ReceiptHandler, Arc, Client

### Community 401 - "clock"
Cohesion: 0.53
Nodes (5): clock(), list_stamp(), local(), DateTime, Option

### Community 402 - "whatsapp-rust backend library"
Cohesion: 0.50
Nodes (5): whatsapp-rust library reference (Bot, messaging, media, pairing, features), whatsapp-rust backend library, whatsapp-rust README, Features: QR pairing, E2E messaging, media, groups, Rust WhatsApp protocol implementation (inspired by whatsmeow, Baileys)

### Community 403 - ".poll"
Cohesion: 0.40
Nodes (4): Context, Output, Pin, Poll

### Community 404 - "install_callbacks"
Cohesion: 0.60
Nodes (4): install_callbacks(), AppWindow, Arc, Controller

### Community 405 - "CachedServerCertChain"
Cohesion: 0.50
Nodes (3): CachedNoiseCert, CachedServerCertChain, From

### Community 406 - "TempBanReason"
Cohesion: 0.50
Nodes (4): Display, Duration, TempBanReason, TemporaryBan

## Knowledge Gaps
- **53 isolated node(s):** `regenerate-proto-desc.sh script`, `MissingTransport`, `MissingHttpClient`, `MissingRuntime`, `BotBuilder<MissingBackend, DefaultTransportState, DefaultHttpState, DefaultRuntimeState>` (+48 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1703 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Result` connect `Result` to `SqliteStore`, `Self`, `mex_operations.rs`, `SignalStore`, `SenderKeyName`, `iq/groups.rs`, `AliceSignalProtocolParameters`, `aes_gcm.rs`, `ProtocolAddress`, `protocol.rs`, `AttrParser<'a>`, `text_msg`, `iq/usync.rs`, `send_receive_benchmark.rs`, `HashState`, `CryptoProviderError`, `Error`, `src/receipt.rs`, `message_processing.rs`, `stanza/call.rs`, `crypto/hash.rs`, `ServiceId`, `PreKeyId`, `iq/tctoken.rs`, `iq/prekeys.rs`, `Vec`, `session_cipher.rs`, `wacore/src/upload.rs`, `messages.rs`, `src/node.rs`, `blocklist.rs`, `wacore/src/download.rs`, `client/lid_pn.rs`, `features/message_edit.rs`, `create_test_client`, `wacore/src/history_sync.rs`, `rust-backend/src/download.rs`, `sticker_pack.rs`, `Runtime`, `NodeStr`, `expand_app_state_keys`, `send_and_expect_text`, `binary_benchmark.rs`, `GenericSignedPreKey`, `PreKeyBundle`, `MexError`, `Device`, `privacy.rs`, `SenderKeyState`, `SignalStoreCache`, `rust-backend/src/handshake.rs`, `encrypt_bot_message`, `rust-backend/src/prekeys.rs`, `Client`, `ratchet/keys.rs`, `props.rs`, `SessionRecord`, `Client`, `signal_adapter.rs`, `state.rs`, `Arc`, `framing.rs`, `GroupInfo`, `iq/contacts.rs`, `features/presence.rs`, `handshake_integration.rs`, `.fmt`, `MockSendContextResolver`, `String`, `BinaryError`, `OwnedNodeRef`, `NodeContent`, `session_divergence.rs`, `stanza/devices.rs`, `.new`, `src/message.rs`, `.new`, `pdo.rs`, `Jid`, `polls.rs`, `companion_reg.rs`, `prepare_dm_retry_stanza`, `TestClient`, `Controller`, `sessions.rs`, `.new`, `encrypt.rs`, `Jid`, `LazyHistorySync`, `rust-backend/src/history_sync.rs`, `Client`, `Client`, `Client`, `.drop`, `.handle_receipt`, `BenchResults`, `decoder.rs`, `wacore/src/pair.rs`, `src/time.rs`, `rust-backend/src/upload.rs`, `features/community.rs`, `types/spam_report.rs`, `tests/media.rs`, `Encoder<'a, W>`, `derive/src/lib.rs`, `zlib_pool.rs`, `iq/mediaconn.rs`, `InvalidSessionError`, `TokioWebSocketTransportFactory`, `HttpClient`, `iq/chatstate.rs`, `src/jid.rs`, `features/mex.rs`, `stanza/business.rs`, `dirty.rs`, `Jid`, `.write_string`, `StoreError`, `prepare_group_stanza`, `xml.rs`, `SessionState`, `MessageContext`, `DeviceListRecord`, `wacore/src/usync.rs`, `wacore/src/prekeys.rs`, `UreqHttpClient`, `WAPatchName`, `features/status.rs`, `Option`, `features/contacts.rs`, `Client`, `Client`, `UploadResponse`, `CapturingMockTransport`, `PublicKey`, `PersistenceManager`, `features/chat_actions.rs`, `AudioPlayer`, `offline_resume.rs`, `setup_session`, `features/events.rs`, `CacheConfig`, `memory_soak.rs`, `labels.rs`, `MediaEncryptor`, `Client`, `NodeRef`, `NoiseSocket`, `PatchList`, `poll.rs`, `src/hash.rs`, `src/ib.rs`, `NodeRef`, `src/message_edit.rs`, `secret_enc_addon.rs`, `PairCodeUtils`, `IqError`, `Client`, `aes_256_cbc_encrypt_into`, `RetryReason`, `ClientError`, `curve.rs`, `GroupInfoResponse`, `PrivateKey`, `SendResult`, `VoiceRecorder`, `bot.rs`, `ChainKey`, `SecretBytes`, `iq/business.rs`, `ResolvedGroupDevices`, `assemble_status_participants`, `waproto/src/lib.rs`, `Downloadable`, `Bot`, `.issue_tokens`, `.get_user_devices`, `CurveError`, `record_helpers.rs`, `state/session.rs`, `prep.rs`, `.send_message`, `Jid`, `enc_handler.rs`, `GatewayImpl`, `ExpandedAppStateKeys`, `CameraSession`, `Client`, `Profile<'a>`, `edge_routing.rs`, `Device`, `create_record_with_previous_sessions`, `Client`, `Client`, `rust-backend/src/pair.rs`, `InfoQuery`, `.reader_from`, `ProtocolNode`, `.pair_with_code`, `generate_reporting_token`, `Self`, `CacheStore`, `scan_message_level`, `prepare_peer_stanza`, `src/reaction.rs`, `comment.rs`, `waproto/build.rs`, `required_attr`, `derive_reporting_token_key`, `iq/keepalive.rs`, `klipy.rs`, `rust-backend/src/version.rs`, `PairCodeState`, `Labels<'a>`, `.send_reaction`, `BenchRng`, `HandshakeUtils`, `main`, `NoiseCipher`, `Receipt`, `DisconnectReason`, `ConnectionOptions`, `noise/src/handshake.rs`, `PreKeyUploadIqNode`, `slint-client/src/main.rs`, `.send_app_state_mutation`?**
  _High betweenness centrality (0.593) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `Result`, `SqliteStore`, `Self`, `mex_operations.rs`, `SignalStore`, `SenderKeyName`, `rust-backend/src/send.rs`, `iq/groups.rs`, `aes_gcm.rs`, `ProtocolAddress`, `MockBackend`, `capturing_client`, `Event`, `text_msg`, `iq/usync.rs`, `send_receive_benchmark.rs`, `send/tests.rs`, `HashState`, `Error`, `message_processing.rs`, `stanza/call.rs`, `ServiceId`, `iq/prekeys.rs`, `session_cipher.rs`, `messages.rs`, `src/node.rs`, `wacore/src/download.rs`, `client/lid_pn.rs`, `rust-backend/src/download.rs`, `proto_helpers.rs`, `stanza/groups.rs`, `sticker_pack.rs`, `send_and_expect_text`, `MexError`, `rust-backend/src/handshake.rs`, `Client`, `props.rs`, `Client`, `.new`, `Arc`, `Store`, `iq/contacts.rs`, `wacore/src/pair_code.rs`, `MockSendContextResolver`, `bridge.rs`, `BinaryError`, `NodeContent`, `stanza/devices.rs`, `pdo.rs`, `Jid`, `polls.rs`, `LidPnCache`, `SessionManager`, `companion_reg.rs`, `prepare_dm_retry_stanza`, `TestClient`, `Controller`, `MsgSecretWriteBuffer`, `encrypt.rs`, `Jid`, `LazyHistorySync`, `rust-backend/src/history_sync.rs`, `Client`, `Client`, `.drop`, `BenchResults`, `decoder.rs`, `Controller`, `wacore/src/pair.rs`, `types/message.rs`, `rust-backend/src/upload.rs`, `features/community.rs`, `types/spam_report.rs`, `tests/media.rs`, `frame.rs`, `Controller`, `derive/src/lib.rs`, `Api`, `iq/mediaconn.rs`, `types/jid.rs`, `views.rs`, `InvalidSessionError`, `UnifiedSessionManager`, `TokioWebSocketTransportFactory`, `HttpClient`, `iq/chatstate.rs`, `GroupMetadata`, `src/jid.rs`, `features/mex.rs`, `Filters`, `stanza/business.rs`, `dirty.rs`, `Jid`, `prepare_group_stanza`, `xml.rs`, `MessageContext`, `DeviceListRecord`, `wacore/src/usync.rs`, `UreqHttpClient`, `MessageInternalFields`, `MessageInfo`, `WAPatchName`, `features/status.rs`, `Option`, `Client`, `ui/tests.rs`, `UploadResponse`, `PrivacyCategory`, `stanza/receipt.rs`, `Node`, `PersistenceManager`, `features/chat_actions.rs`, `AudioPlayer`, `features/events.rs`, `NodeRef`, `PatchList`, `util/jid.rs`, `Option`, `src/ib.rs`, `NodeRef`, `PairCodeUtils`, `Persist`, `BotBuilder<B, T, H, R>`, `IqError`, `ClientProfile`, `Client`, `controller.rs`, `RetryReason`, `Message`, `GroupInfoResponse`, `SendResult`, `VoiceRecorder`, `bot.rs`, `Vec`, `SenderKeyDeviceCache`, `iq/business.rs`, `Downloadable`, `.issue_tokens`, `DevicePropsOverride`, `EnforcementExtraData`, `Server`, `prep.rs`, `DownloadParams`, `enc_handler.rs`, `GatewayImpl`, `CameraSession`, `SignalProtocolError`, `build_reaction_message`, `Device`, `ImageCache`, `rust-backend/src/pair.rs`, `InfoQuery`, `.pair_with_code`, `Option`, `PairUtils`, `ApiError`, `prepare_peer_stanza`, `wacore/src/store/error.rs`, `waproto/build.rs`, `history_sync_benchmark.rs`, `Settings`, `rust-backend/src/main.rs`, `required_attr`, `klipy.rs`, `PairCodeState`, `attach.rs`, `main`, `Receipt`, `DisconnectReason`, `MessageStanza`, `ReceiptType`, `noise/src/handshake.rs`, `clock`?**
  _High betweenness centrality (0.341) - this node is a cross-community bridge._
- **Why does `Error` connect `Error` to `SqliteStore`, `iq/usync.rs`, `send_receive_benchmark.rs`, `HashState`, `src/receipt.rs`, `message_processing.rs`, `ServiceId`, `iq/tctoken.rs`, `iq/prekeys.rs`, `wacore/src/download.rs`, `client/lid_pn.rs`, `create_test_client`, `wacore/src/history_sync.rs`, `rust-backend/src/download.rs`, `sticker_pack.rs`, `NodeStr`, `MexError`, `privacy.rs`, `rust-backend/src/handshake.rs`, `rust-backend/src/prekeys.rs`, `Client`, `props.rs`, `framing.rs`, `GroupInfo`, `iq/contacts.rs`, `features/presence.rs`, `.fmt`, `String`, `BinaryError`, `NodeContent`, `src/message.rs`, `pdo.rs`, `sessions.rs`, `encrypt.rs`, `Jid`, `LazyHistorySync`, `rust-backend/src/history_sync.rs`, `Client`, `Client`, `Client`, `.handle_receipt`, `wacore/src/pair.rs`, `features/community.rs`, `types/spam_report.rs`, `derive/src/lib.rs`, `iq/mediaconn.rs`, `TokioWebSocketTransportFactory`, `HttpClient`, `iq/chatstate.rs`, `features/mex.rs`, `dirty.rs`, `MessageContext`, `wacore/src/prekeys.rs`, `WAPatchName`, `features/status.rs`, `Option`, `Client`, `Client`, `CapturingMockTransport`, `PublicKey`, `MediaEncryptor`, `Client`, `NodeRef`, `IqError`, `Client`, `RetryReason`, `PrivateKey`, `SendResult`, `bot.rs`, `iq/business.rs`, `.issue_tokens`, `.get_user_devices`, `CurveError`, `record_helpers.rs`, `Jid`, `Client`, `SignalProtocolError`, `Device`, `Client`, `rust-backend/src/pair.rs`, `InfoQuery`, `ApiError`, `wacore/src/store/error.rs`, `required_attr`, `iq/keepalive.rs`, `.send_reaction`, `BenchRng`, `main`, `Receipt`, `ConnectionOptions`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **What connects `regenerate-proto-desc.sh script`, `MissingTransport`, `MissingHttpClient` to the rest of the system?**
  _53 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Result` be split into smaller, more focused modules?**
  _Cohesion score 0.022080386406762118 - nodes in this community are weakly interconnected._
- **Should `SqliteStore` be split into smaller, more focused modules?**
  _Cohesion score 0.0365105491545713 - nodes in this community are weakly interconnected._
- **Should `Self` be split into smaller, more focused modules?**
  _Cohesion score 0.0693815987933635 - nodes in this community are weakly interconnected._