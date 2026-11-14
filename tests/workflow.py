"""Running application checks; only the Venice HTTP boundary is substituted."""
import base64, struct, zlib, hashlib, http.server, json, os, pathlib, socket, subprocess, tempfile, threading, time, unittest, urllib.request, urllib.error, uuid
ROOT = pathlib.Path(__file__).resolve().parents[1]
def chunk(kind, data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
PNG = b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',1,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\x00\xff\x00\x00\xff'))+chunk(b'IEND',b'')

def port():
    with socket.socket() as s:
        s.bind(('127.0.0.1',0)); return s.getsockname()[1]
class Provider(http.server.BaseHTTPRequestHandler):
    requests = []
    response = {'images':[base64.b64encode(PNG).decode()], 'seed':42}
    delay = 0
    status = 200
    response_headers = {}
    catalog_status = 200
    raw_body = None
    def log_message(self,*args): pass
    def do_GET(self):
        data={'data':[{'id':'chroma','model_spec':{'name':'Chroma','offline':False,'privacy':'private','uncensored':True,'pricing':{'generation':{'usd':0.02}},'constraints':{'promptCharacterLimit':5000,'widthHeightDivisor':16}}}]}
        self.send_response(self.catalog_status);self.end_headers();self.wfile.write(json.dumps(data).encode())
    def do_POST(self):
        self.requests.append((self.headers.get('Authorization'),json.loads(self.rfile.read(int(self.headers['Content-Length'])))))
        time.sleep(self.delay)
        self.send_response(self.status);self.send_header('Content-Type','application/json')
        for key,value in self.response_headers.items(): self.send_header(key,value)
        self.end_headers()
        try:self.wfile.write(self.raw_body if self.raw_body is not None else json.dumps(self.response).encode())
        except BrokenPipeError:pass
class Workflow(unittest.TestCase):
    def setUp(self):
        Provider.requests=[];Provider.delay=0;Provider.status=200;Provider.catalog_status=200;Provider.response_headers={};Provider.raw_body=None;Provider.response={'images':[base64.b64encode(PNG).decode()],'seed':42}
        self.tmp=tempfile.TemporaryDirectory(prefix='venice-test-',dir='/var/tmp');self.library=pathlib.Path(self.tmp.name)/'library'
        self.provider=http.server.ThreadingHTTPServer(('127.0.0.1',0),Provider)
        threading.Thread(target=self.provider.serve_forever,daemon=True).start();self.start()
    def start(self):
        self.url=f'http://127.0.0.1:{port()}'
        env={**os.environ,'VENICE_IMAGE_STUDIO_DATA_DIR':str(self.library),'VENICE_API_KEY':'must-never-leave-test-mode',**getattr(self,'extra_env',{})}
        self.log=open(pathlib.Path(self.tmp.name)/'process.log','ab')
        self.process=subprocess.Popen([str(ROOT/'target/debug/venice-image-studio'),'--port',self.url.rsplit(':',1)[1],'--fixture-provider',f'http://127.0.0.1:{self.provider.server_port}'],cwd=ROOT,env=env,stdout=self.log,stderr=self.log)
        for _ in range(100):
            try:self.state=self.call('/api/state');return
            except (OSError,ValueError):time.sleep(.05)
        raise AssertionError('Application did not start: '+(pathlib.Path(self.tmp.name)/'process.log').read_text())
    def stop(self):self.process.terminate();self.process.wait(timeout=5);self.log.close()
    def tearDown(self):
        if hasattr(self,'process') and self.process.poll() is None:self.stop()
        if hasattr(self,'log') and not self.log.closed:self.log.close()
        self.provider.shutdown();self.provider.server_close();self.tmp.cleanup()
    def call(self,path,body=None,method=None,headers=None,raw=False):
        h={'Origin':self.url,'X-Studio-Token':getattr(self,'state',{}).get('token','')};h.update(headers or {})
        if body is not None:h['Content-Type']='application/json'
        if path=='/api/generate' and body is not None:body={'catalog_fetched_at':self.state['catalog']['fetched_at'],**body}
        req=urllib.request.Request(self.url+path,data=json.dumps(body).encode() if body is not None else None,headers=h,method=method)
        try:
            with urllib.request.urlopen(req,timeout=5) as r:return r.read() if raw else json.load(r)
        except urllib.error.HTTPError as error:
            payload=error.read();error.close();error.read=lambda:payload
            raise
    def draft(self,prompt='Synthetic test prompt'):
        d={'id':str(uuid.uuid4()),'prompt':prompt,'safe_mode':False,'model':'chroma','parent_id':None}
        self.call('/api/draft',d,'PUT');return d
    def finish(self,id):
        for _ in range(100):
            a=self.call('/api/attempts/'+id)
            if a['status'] not in ['dispatching','waiting']:return a
            time.sleep(.05)
        self.fail('Attempt did not finish')
    def test_long_prompt_original_recipe_restart_and_reuse(self):
        self.assertIn(b'Writing desk',self.call('/',raw=True))
        prompt='A synthetic scene: '+('é🦊\n'*600)+'  exact ending\n'
        d=self.draft(prompt)
        a=self.call('/api/generate',{'draft_id':d['id']});a=self.finish(a['id'])
        self.assertEqual(a['status'],'saved')
        auth,outbound=Provider.requests[0]
        self.assertEqual(auth,'Bearer offline-test')
        self.assertEqual(outbound,{'model':'chroma','prompt':prompt,'width':1024,'height':1024,'variants':1,'safe_mode':False,'enhance_prompt':False,'return_binary':False,'embed_exif_metadata':False,'format':'png'})
        image=self.call('/api/images/'+a['results'][0]['id'],raw=True)
        self.assertEqual(image,PNG);self.assertEqual(a['results'][0]['sha256'],hashlib.sha256(PNG).hexdigest())
        self.assertEqual(a['results'][0]['width'],1);self.assertEqual(a['prompt'],prompt)
        recipe=self.call('/api/attempts/'+a['id'])
        self.stop();self.start()
        self.assertEqual(self.call('/api/attempts/'+a['id']),recipe)
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
        self.assertEqual(reused['prompt'],prompt);self.assertEqual(reused['parent_id'],a['id'])
        self.assertEqual(len(Provider.requests),1)
        self.assertEqual(self.call('/api/attempts/'+a['id']),recipe)
    def test_provider_cannot_echo_credential_in_recipe(self):
        Provider.status=422
        Provider.response={'error':{'message':'invalid offline-test', 'offline-test':'secret name', 'api_key':'offline-test'},'seed':'offline-test'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertNotIn('offline-test',json.dumps(a))
        self.assertNotIn('offline-test',json.dumps(self.call('/api/state')))
        self.assertEqual(a['status'],'rejected')
        self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])
    def test_duplicate_reload_and_interrupted_dispatch_are_not_replayed(self):
        Provider.delay=2
        d=self.draft();a=self.call('/api/generate',{'draft_id':d['id']})
        for _ in range(50):
            if Provider.requests:break
            time.sleep(.02)
        again=self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(again['id'],a['id'])
        self.assertEqual(self.call('/api/state')['inflight'],a['id'])
        self.stop();self.start()
        recovered=self.call('/api/attempts/'+a['id'])
        self.assertEqual(recovered['status'],'indeterminate')
        self.assertEqual(recovered['billing']['charge'],'unknown')
        self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(len(Provider.requests),1)

    def test_late_save_failure_keeps_download_and_retry_never_generates(self):
        Provider.delay=.5
        d=self.draft();a=self.call('/api/generate',{'draft_id':d['id']})
        self.library.chmod(0o500)
        try:
            a=self.finish(a['id'])
            self.assertEqual(a['status'],'save_failed')
            self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        finally:self.library.chmod(0o700)
        a=self.call('/api/attempts/'+a['id']+'/save',{})
        self.assertEqual(a['status'],'saved');self.assertEqual(len(Provider.requests),1)
        self.stop();self.start()
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)

    def test_origin_token_and_library_identity_boundaries(self):
        for headers in [{'Origin':'https://other.example'},{'Host':'other.example'},{'Sec-Fetch-Site':'cross-site'}]:
            with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/state',headers=headers)
            self.assertEqual(failure.exception.code,403)
        with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/generate',{'draft_id':str(uuid.uuid4())},headers={'X-Studio-Token':'wrong'})
        self.assertEqual(failure.exception.code,403)
        for path in ['/api/images/..%2F.env','/api/images/'+str(uuid.uuid4())]:
            with self.assertRaises(urllib.error.HTTPError):self.call(path)
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        result=a['results'][0]
        original=self.library/result['relative_path']
        replacement=pathlib.Path(self.tmp.name)/'elsewhere.png';replacement.write_bytes(PNG)
        original.unlink();original.symlink_to(replacement)
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/images/'+result['id'],raw=True)
        self.assertEqual(self.library.stat().st_mode&0o777,0o700)
        self.assertEqual((self.library/(a['id']+'.recipe.json')).stat().st_mode&0o777,0o600)

    def test_response_flags_partial_images_and_stale_catalog_remain_inspectable(self):
        Provider.response={'images':[base64.b64encode(PNG).decode(),'not base64',base64.b64encode(PNG).decode()], 'content_policy_violation_count':1,'suggested_prompt':'<script>untrusted</script>'}
        Provider.response_headers={'x-venice-is-blurred':'true','x-venice-is-content-violation':'true','x-venice-content-violation-count':'1','x-venice-enhanced-prompt':'unexpected%20revision'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(len(a['results']),2)
        for r in a['results']:self.assertEqual(self.call('/api/images/'+r['id'],raw=True),PNG)
        self.assertEqual(a['provider']['headers']['x-venice-is-blurred'],'true')
        self.assertEqual(a['provider']['headers']['x-venice-is-content-violation'],'true')
        self.assertEqual(a['provider']['headers']['x-venice-enhanced-prompt'],'unexpected%20revision')
        self.assertEqual(a['provider']['body']['suggested_prompt'],'<script>untrusted</script>')
        self.assertNotIn('images',a['provider']['body'])
        previous=self.call('/api/state')['catalog'];Provider.catalog_status=503
        refreshed=self.call('/api/catalog/refresh',{})
        self.assertIsNotNone(refreshed['catalog_error']);self.assertEqual(refreshed['catalog'],previous)
        self.assertEqual(len(Provider.requests),1)

    def test_redirects_and_invalid_images_never_trigger_fallback(self):
        Provider.status=302;Provider.response_headers={'Location':f'http://127.0.0.1:{self.provider.server_port}/elsewhere'}
        Provider.response={'error':'inspect this suggestion', 'retryable':True, 'recommended_model':'another-model'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'failed');self.assertEqual(len(Provider.requests),1)
        Provider.status=200;Provider.response_headers={};Provider.raw_body=None;Provider.response={'images':['aW52YWxpZA==']}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'failed');self.assertEqual(a['results'],[]);self.assertEqual(len(Provider.requests),2)
    def test_save_retry_preserves_known_rejection_outcome(self):
        Provider.delay=.3;Provider.status=422;Provider.response={'error':'Invalid settings','retryable':False}
        d=self.draft();a=self.call('/api/generate',{'draft_id':d['id']})
        self.library.chmod(0o500)
        try:
            a=self.finish(a['id']);self.assertEqual(a['status'],'save_failed')
        finally:self.library.chmod(0o700)
        saved=self.call('/api/attempts/'+a['id']+'/save',{})
        self.assertEqual(saved['status'],'rejected')
        self.assertEqual(saved['provider']['body']['error'],'Invalid settings')
        self.assertEqual(len(Provider.requests),1)
    def test_crash_between_original_and_recipe_recovers_without_replay(self):
        self.stop();self.extra_env={'VENICE_STUDIO_TEST_CRASH_AFTER_ORIGINAL':'1'};self.start()
        d=self.draft();a=self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(self.process.wait(timeout=2),86)
        self.log.close();self.extra_env={};self.start()
        recovered=self.call('/api/attempts/'+a['id'])
        self.assertEqual(recovered['status'],'saved')
        self.assertEqual(self.call('/api/images/'+recovered['results'][0]['id'],raw=True),PNG)
        self.assertEqual(len(Provider.requests),1)

    def test_incomplete_body_retains_status_identity_and_unknown_outcome(self):
        Provider.raw_body=b'{"images":["truncated'
        Provider.response_headers={'Content-Length':'1000','x-request-id':'synthetic-incomplete'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'indeterminate')
        self.assertEqual(a['provider']['http_status'],200)
        self.assertEqual(a['provider']['headers']['x-request-id'],'synthetic-incomplete')
        self.assertTrue(a['provider']['body']['body_incomplete'])
        self.assertEqual(len(Provider.requests),1)
    def test_missing_credentials_preserve_draft_and_prevent_dispatch(self):
        self.stop();self.extra_env={'VENICE_STUDIO_TEST_MISSING_CREDENTIAL':'1'};self.start()
        self.assertFalse(self.state['credential_present'])
        d=self.draft()
        with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/generate',{'draft_id':d['id']})
        self.assertIn('VENICE_API_KEY',failure.exception.read().decode())
        self.assertEqual(len(Provider.requests),0)
        self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])

    def test_setup_rejects_relative_symlink_and_git_worktree_storage(self):
        self.stop()
        worktree=pathlib.Path(self.tmp.name)/'worktree';worktree.mkdir();(worktree/'.git').write_text('gitdir: synthetic-linked-worktree')
        target=pathlib.Path(self.tmp.name)/'outside';target.mkdir()
        symlink=pathlib.Path(self.tmp.name)/'link';symlink.symlink_to(target,target_is_directory=True)
        for path,word in [(worktree/'library','Git'),(symlink/'library','symbolic'),(pathlib.Path('relative'),'absolute')]:
            self.library=path;self.start()
            self.assertIn(word,self.state['setup_error']);self.assertIsNone(self.state['library'])
            self.stop()
        self.assertEqual(len(Provider.requests),0)

    def test_prompt_body_and_image_bounds_fail_without_hidden_dispatch(self):
        d=self.draft('🦊'*5001)
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(len(Provider.requests),0)
        with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/draft',{'prompt':'x'*270000},'PUT')
        self.assertEqual(failure.exception.code,413)
        self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])
        Provider.response_headers={'Content-Length':str(129*1024*1024)};Provider.raw_body=b''
        d=self.draft('🦊'*5000);a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'indeterminate');self.assertTrue(a['provider']['body']['body_incomplete'])
        self.assertIn('limit',a['provider']['body']['capture_error'])
        self.assertEqual(Provider.requests[0][1]['prompt'],d['prompt'])
        huge=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',8193,8193,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\x00'))+chunk(b'IEND',b'')
        Provider.response_headers={};Provider.raw_body=None;Provider.response={'images':[base64.b64encode(huge).decode()]}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['results'],[]);self.assertEqual(a['status'],'failed');self.assertEqual(len(Provider.requests),2)

    def test_access_failure_is_actionable_and_logs_remain_private(self):
        Provider.status=401;Provider.response={'error':'offline-test rejected'}
        d=self.draft('private synthetic text');a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'access_error');self.assertIn('private API key',a['summary'])
        self.assertNotIn('offline-test',json.dumps(a))
        log=(pathlib.Path(self.tmp.name)/'process.log').read_text()
        self.assertNotIn(d['prompt'],log);self.assertNotIn('offline-test',log)
    def test_generate_requires_the_estimate_revision_the_browser_inspected(self):
        d=self.draft()
        seen=self.state['catalog']['fetched_at']
        self.call('/api/catalog/refresh',{})
        with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/generate',{'draft_id':d['id'],'catalog_fetched_at':seen})
        self.assertEqual(failure.exception.code,409)
        self.assertEqual(len(Provider.requests),0)
if __name__=='__main__':unittest.main()
