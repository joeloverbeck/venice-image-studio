"""Joined Writing desk checks through the browser and actual Rust application."""
import base64
import json
import os
import pathlib
import subprocess
import struct
import unittest
import zlib
import workflow as seam


class WritingDesk(unittest.TestCase):
    setUp = seam.Workflow.setUp
    tearDown = seam.Workflow.tearDown
    start = seam.Workflow.start
    stop = seam.Workflow.stop
    call = seam.Workflow.call
    draft = seam.Workflow.draft
    finish = seam.Workflow.finish

    def browser(self, scenario, **values):
        env = {**os.environ, 'STUDIO_URL': self.url, 'SCENARIO': scenario, **values}
        subprocess.run(['node', str(pathlib.Path(__file__).with_suffix('.mjs'))], env=env, check=True)

    def test_keyboard_model_selection_and_controls_keep_focus(self):
        self.browser('keyboard')
        self.assertEqual(seam.Provider.requests, [])

    def test_numeric_draft_edit_survives_reload_before_blur(self):
        self.browser('draft-number')
        self.assertEqual(seam.Provider.requests, [])

    def test_advanced_text_and_seed_edits_preserve_draft_and_validation(self):
        seam.Provider.catalog = json.loads((seam.ROOT / 'docs/research/models-2026-11-14.json').read_text())['models']
        self.call('/api/catalog/refresh', {})
        self.browser('draft-advanced')
        self.assertEqual(seam.Provider.requests, [])

    def test_saved_status_waits_for_the_current_edit(self):
        self.browser('draft-status')
        self.assertEqual(seam.Provider.requests, [])

    def test_proportional_resolution_reaches_request_and_restart_reuse(self):
        self.browser('pixel-resolution')
        request = seam.Provider.requests[0][1]
        self.assertEqual((request['width'], request['height']), (960, 1280))
        self.assertNotIn('aspect_ratio', request)
        self.assertNotIn('resolution', request)
        attempt = self.call('/api/state')['attempts'][0]
        self.assertEqual(attempt['request'], request)
        self.stop()
        self.start()
        reused = self.call('/api/attempts/' + attempt['id'] + '/reuse', {})
        self.assertEqual((reused['settings']['width'], reused['settings']['height']), (960, 1280))
        self.browser('pixel-resolution-reused')
        self.assertEqual(len(seam.Provider.requests), 1)

    def test_changed_divisor_retains_incompatible_draft_and_offers_legal_sizes(self):
        draft = self.draft('Synthetic changed size constraint')
        draft['settings'] = {'width': 1280, 'height': 720}
        self.call('/api/draft', draft, 'PUT')
        seam.Provider.catalog = [{'id': 'chroma', 'model_spec': {
            'name': 'Chroma', 'offline': False, 'pricing': {'generation': {'usd': 0.02}},
            'constraints': {'promptCharacterLimit': 5000, 'widthHeightDivisor': 64}}}]
        self.call('/api/catalog/refresh', {})
        self.browser('pixel-divisor')
        for divisor in [None, 0, 1281, 2**64 - 1]:
            seam.Provider.catalog[0]['model_spec']['constraints']['widthHeightDivisor'] = divisor
            self.call('/api/catalog/refresh', {})
            state = self.call('/api/state')
            self.assertEqual(state['preflight']['contract']['pixel_presets'], [])
            self.assertTrue(state['preflight']['errors'])
        self.assertEqual(seam.Provider.requests, [])

    def test_waiting_status_remains_visible_during_library_inspection(self):
        draft = self.draft('Earlier saved image')
        earlier = self.finish(self.call('/api/generate', {'draft_id': draft['id']})['id'])
        seam.Provider.delay = 3
        self.browser('waiting', EARLIER_ID=earlier['id'])
        self.assertEqual(len(seam.Provider.requests), 2)
        self.assertEqual(seam.Provider.requests[-1][1]['prompt'], 'A new explicit attempt 🦊')

    def test_save_failure_stays_reachable_and_retry_keeps_keyboard_focus(self):
        draft = self.draft('Earlier saved image')
        earlier = self.finish(self.call('/api/generate', {'draft_id': draft['id']})['id'])
        seam.Provider.delay = .5
        draft = self.draft('Keep these received bytes')
        failed = self.call('/api/generate', {'draft_id': draft['id']})
        self.library.chmod(0o500)
        try:
            failed = self.finish(failed['id'])
            self.assertEqual(failed['status'], 'save_failed')
        finally:
            self.library.chmod(0o700)
        self.browser('saving', EARLIER_ID=earlier['id'])
        self.assertEqual(len(seam.Provider.requests), 2)
        self.assertEqual(self.call('/api/attempts/' + failed['id'])['status'], 'saved')

    def test_narrow_window_navigation_and_large_original_inspection(self):
        original = b'\x89PNG\r\n\x1a\n' + seam.chunk(b'IHDR', struct.pack('>IIBBBBB', 4096, 4096, 8, 6, 0, 0, 0)) + seam.chunk(b'IDAT', zlib.compress((b'\0' + b'\x90\x30\x80\xff' * 4096) * 4096)) + seam.chunk(b'IEND', b'')
        seam.Provider.response = {'images': [base64.b64encode(original).decode()]}
        draft = self.draft('A large original for inspection')
        self.finish(self.call('/api/generate', {'draft_id': draft['id']})['id'])
        self.browser('responsive')
        self.assertEqual(len(seam.Provider.requests), 1)


if __name__ == '__main__':
    unittest.main()
