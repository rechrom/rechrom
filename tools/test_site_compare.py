#!/usr/bin/env python3

import unittest

from site_compare import trace_timing


def event(name, start_ns, *, generation=None, has_content=None):
    fields = {}
    if generation is not None:
        fields["document_generation"] = generation
    if has_content is not None:
        fields["has_content"] = has_content
    return {
        "name": name,
        "start_ns": start_ns,
        "duration_ns": 0,
        "context": {"target_id": 1},
        "fields": fields,
    }


class TraceTimingTest(unittest.TestCase):
    def test_uses_request_for_final_document_not_startup_about_blank(self):
        snapshot = {
            "spans": [
                event("OpenRequest", 10_000_000),
                event("OpenRequest", 40_000_000),
            ],
            "instants": [
                event("DocumentCommitted", 20_000_000, generation=1),
                event("OffscreenFramePublished", 30_000_000, generation=1, has_content=1),
                event("DocumentCommitted", 45_000_000, generation=2),
                event("DocumentPaintArtifact", 55_000_000, generation=2),
                event("OffscreenFramePublished", 65_000_000, generation=2, has_content=1),
            ],
        }

        timing = trace_timing(snapshot)

        self.assertEqual(timing["committed_ms"], 5.0)
        self.assertEqual(timing["first_artifact_ms"], 15.0)
        self.assertEqual(timing["first_offscreen_frame_ms"], 25.0)

    def test_ignores_request_after_selected_commit(self):
        snapshot = {
            "spans": [
                event("OpenRequest", 10_000_000),
                event("OpenRequest", 50_000_000),
            ],
            "instants": [
                event("DocumentCommitted", 20_000_000, generation=7),
                event("DocumentPaintArtifact", 25_000_000, generation=7),
                event("OffscreenFramePublished", 30_000_000, generation=7, has_content=1),
            ],
        }

        timing = trace_timing(snapshot)

        self.assertEqual(timing["committed_ms"], 10.0)
        self.assertEqual(timing["first_offscreen_frame_ms"], 20.0)


if __name__ == "__main__":
    unittest.main()
