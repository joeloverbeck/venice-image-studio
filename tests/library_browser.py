"""Browser check of the same running app; set PLAYWRIGHT_MODULE if not installed locally."""
import base64
import json
import os
import pathlib
import struct
import subprocess
import unittest
import zlib
import workflow as seam


class LibraryBrowser(unittest.TestCase):
    setUp = seam.Workflow.setUp
    tearDown = seam.Workflow.tearDown
    start = seam.Workflow.start
    stop = seam.Workflow.stop
    call = seam.Workflow.call
    draft = seam.Workflow.draft
    finish = seam.Workflow.finish

    def test_original_inspection_export_reuse_and_draft_navigation(self):
        original = b'\x89PNG\r\n\x1a\n' + seam.chunk(b'IHDR', struct.pack('>IIBBBBB', 2048, 1024, 8, 6, 0, 0, 0)) + seam.chunk(b'IDAT', zlib.compress((b'\0' + b'\x90\x30\x80\xff' * 2048) * 1024)) + seam.chunk(b'IEND', b'')
        seam.Provider.response = {'images': [base64.b64encode(original).decode()], 'seed': 42,
                                  'private_detail': 'PRIVATE DIAGNOSTIC MARKER'}
        d = self.draft('Amber original <script>window.unsafe=true</script> 🦊')
        attempt = self.finish(self.call('/api/generate', {'draft_id': d['id']})['id'])
        history = (self.library / (attempt['id'] + '.recipe.json')).read_bytes()
        self.draft('Unfinished draft stays here')
        self.stop()
        seam.Provider.catalog = []
        self.start()
        env = {**os.environ, 'STUDIO_URL': self.url, 'ATTEMPT_ID': attempt['id'],
               'ORIGINAL_SHA256': attempt['results'][0]['sha256']}
        subprocess.run(['node', str(pathlib.Path(__file__).with_suffix('.mjs'))], env=env, check=True)
        self.assertEqual(len(seam.Provider.requests), 1)
        self.assertEqual((self.library / (attempt['id'] + '.recipe.json')).read_bytes(), history)


if __name__ == '__main__':
    unittest.main()
