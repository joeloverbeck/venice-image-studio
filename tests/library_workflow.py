"""Saved-work behavior through the running local API and synthetic Venice HTTP."""
import base64
import json
import unittest
import urllib.error
import urllib.request
import uuid
import workflow as seam
from workflow import Provider, PNG


class SavedWork(unittest.TestCase):
    setUp = seam.Workflow.setUp
    tearDown = seam.Workflow.tearDown
    start = seam.Workflow.start
    stop = seam.Workflow.stop
    call = seam.Workflow.call
    draft = seam.Workflow.draft
    finish = seam.Workflow.finish

    def test_search_after_restart_keeps_failed_partial_and_flagged_attempts(self):
        expected = []
        for prompt, status, response, headers in [
            ('Amber fox exact phrase', 200, {'images': [base64.b64encode(PNG).decode()]}, {}),
            ('Cobalt rainy street', 422, {'error': {'message': 'Invalid synthetic setting'}}, {}),
            ('Amber bird incomplete', 200, {'images': [base64.b64encode(PNG).decode(), 'corrupt']}, {}),
            ('Magenta flagged scene', 200, {'images': [base64.b64encode(PNG).decode()]}, {'x-venice-is-blurred': 'true'}),
        ]:
            Provider.status, Provider.response, Provider.response_headers = status, response, headers
            d = self.draft(prompt)
            expected.append(self.finish(self.call('/api/generate', {'draft_id': d['id']})['id']))
        self.stop()
        self.start()
        all_entries = self.call('/api/library')['attempts']
        self.assertEqual([a['id'] for a in all_entries], [a['id'] for a in reversed(expected)])
        self.assertEqual([a['id'] for a in self.call('/api/library?order=oldest')['attempts']], [a['id'] for a in expected])
        self.assertEqual([a['id'] for a in self.call('/api/library?search=AMBER')['attempts']], [expected[2]['id'], expected[0]['id']])
        self.assertEqual(len(self.call('/api/library?search=CHROMA')['attempts']), 4)
        self.assertEqual(self.call('/api/library?search=absent')['attempts'], [])
        self.assertEqual([a['outcome'] for a in all_entries], [a['outcome'] for a in reversed(expected)])
        self.assertEqual([a['status'] for a in all_entries], [a['status'] for a in reversed(expected)])
        self.assertEqual(len(Provider.requests), 4)

    def test_recipe_export_is_explicit_and_separate_from_diagnostics(self):
        Provider.response = {'images': [base64.b64encode(PNG).decode()], 'seed': 42,
                             'provider_internal_notes': 'PRIVATE DIAGNOSTIC MARKER',
                             'api_key': 'offline-test'}
        d = self.draft('Exact authored export prompt <script>bad()</script> 🦊\n')
        a = self.finish(self.call('/api/generate', {'draft_id': d['id']})['id'])
        before = (self.library / (a['id'] + '.recipe.json')).read_bytes()
        self.stop()
        self.start()
        path = '/api/attempts/' + a['id'] + '/recipe'
        exported = self.call(path)
        self.assertEqual(exported['prompt'], d['prompt'])
        self.assertEqual(exported['request'], a['request'])
        self.assertEqual(exported['requested_seed'], None)
        self.assertEqual(exported['returned_seed'], 42)
        self.assertEqual(exported['estimate'], a['estimate'])
        self.assertEqual(exported['model']['id'], 'chroma')
        self.assertEqual(exported['results'][0]['sha256'], a['results'][0]['sha256'])
        self.assertNotIn('provider', exported)
        self.assertNotIn('PRIVATE DIAGNOSTIC MARKER', json.dumps(exported))
        self.assertNotIn('offline-test', json.dumps(exported))
        with urllib.request.urlopen(self.url + path) as response:
            self.assertIn('attachment;', response.headers['Content-Disposition'])
            self.assertEqual(response.headers['Content-Type'], 'application/json')
            self.assertEqual(response.headers['Cache-Control'], 'no-store')
        for unknown in [str(uuid.uuid4()), '%2e%2e']:
            with self.assertRaises(urllib.error.HTTPError):
                self.call('/api/attempts/' + unknown + '/recipe')
        with self.assertRaises(urllib.error.HTTPError) as denied:
            self.call(path, headers={'Origin': 'https://untrusted.invalid'})
        self.assertEqual(denied.exception.code, 403)
        self.assertEqual((self.library / (a['id'] + '.recipe.json')).read_bytes(), before)
        self.assertEqual(len(Provider.requests), 1)

    def test_removed_model_reuse_preserves_lineage_and_history_without_dispatch(self):
        d = self.draft('Historical choice remains identifiable')
        a = self.finish(self.call('/api/generate', {'draft_id': d['id']})['id'])
        recipe = (self.library / (a['id'] + '.recipe.json')).read_bytes()
        self.stop()
        Provider.catalog = []
        self.start()
        copy = self.call('/api/attempts/' + a['id'] + '/reuse', {})
        self.assertNotEqual(copy['id'], a['id'])
        self.assertEqual(copy['parent_id'], a['id'])
        self.assertEqual(copy['model'], 'chroma')
        self.assertEqual(copy['prompt'], d['prompt'])
        state = self.call('/api/state')
        self.assertTrue(any('unavailable' in reason for reason in state['preflight']['errors']))
        copy['prompt'] = 'An independently edited new draft'
        copy['settings']['width'] = 768
        self.call('/api/draft', copy, 'PUT')
        self.stop()
        self.start()
        self.assertEqual(self.call('/api/state')['draft'], copy)
        self.assertEqual((self.library / (a['id'] + '.recipe.json')).read_bytes(), recipe)
        self.assertEqual(self.call('/api/images/' + a['results'][0]['id'], raw=True), PNG)
        self.assertEqual(len(Provider.requests), 1)

    def test_unrecognized_historical_settings_do_not_silently_change_draft(self):
        d = self.draft('Future settings remain in history')
        a = self.finish(self.call('/api/generate', {'draft_id': d['id']})['id'])
        pending = self.draft('Unfinished current draft')
        self.stop()
        path = self.library / (a['id'] + '.recipe.json')
        future = json.loads(path.read_text())
        future['request']['future_setting'] = {'mode': 'intent must not disappear'}
        path.write_text(json.dumps(future))
        before = path.read_bytes()
        self.start()
        with self.assertRaises(urllib.error.HTTPError) as unavailable:
            self.call('/api/attempts/' + a['id'] + '/reuse', {})
        self.assertIn(b'cannot represent', unavailable.exception.read())
        self.assertEqual(self.call('/api/state')['draft']['prompt'], pending['prompt'])
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(len(Provider.requests), 1)


if __name__ == '__main__':
    unittest.main()
