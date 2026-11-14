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
    pricing = {'generation':{'usd':0.02}}
    raw_body = None
    catalog = None
    styles_status = 200
    styles = ['3D Model','Analog Film']
    def log_message(self,*args): pass
    def do_GET(self):
        if self.path == '/image/styles':
            self.send_response(self.styles_status);self.end_headers();self.wfile.write(json.dumps({'data':self.styles}).encode());return
        data={'data':[{'id':'chroma','model_spec':{'name':'Chroma','offline':False,'privacy':'private','uncensored':True,'pricing':self.pricing,'constraints':{'promptCharacterLimit':5000,'widthHeightDivisor':16}}}]}
        if self.catalog is not None:data={'data':self.catalog}
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
        Provider.pricing={'generation':{'usd':0.02}}
        Provider.catalog=None
        Provider.styles_status=200;Provider.styles=['3D Model','Analog Film']
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
        if path=='/api/generate' and body is not None:
            inspected=self.call('/api/state')
            body={'catalog_fetched_at':self.state['catalog']['fetched_at'],'draft_revision':inspected['preflight']['draft_revision'],**body}
        req=urllib.request.Request(self.url+path,data=json.dumps(body,ensure_ascii=False).encode() if body is not None else None,headers=h,method=method)
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

    def test_variants_price_request_partial_originals_and_reuse(self):
        d=self.draft();d['settings']={'variants':3}
        self.call('/api/draft',d,'PUT');self.state=self.call('/api/state')
        self.assertEqual(self.state['estimate']['usd'],0.06)
        Provider.response={'images':[base64.b64encode(PNG).decode(),base64.b64encode(PNG).decode()], 'content_policy_violation_count':1}
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(Provider.requests[0][1]['variants'],3)
        self.assertEqual(a['request']['variants'],3)
        self.assertEqual(a['outcome'],'partial')
        self.assertIn('2 of 3',a['summary']);self.assertIn('1 omitted',a['summary'])
        self.assertEqual(len(a['results']),2)
        for r in a['results']:self.assertEqual(self.call('/api/images/'+r['id'],raw=True),PNG)
        self.stop();self.start()
        self.assertEqual(self.call('/api/attempts/'+a['id']),a)
        reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
        self.assertEqual(reused['settings']['variants'],3)
        reused['settings']['variants']=5
        self.call('/api/draft',reused,'PUT')
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/generate',{'draft_id':reused['id']})
        self.assertEqual(len(Provider.requests),1)

    def test_ratio_quality_selection_reaches_request_and_durable_reuse(self):
        Provider.catalog=[{'id':'grok-imagine-image-2-0','model_spec':{
            'name':'Grok Imagine 2.0','offline':False,'privacy':'private',
            'constraints':{'promptCharacterLimit':7500,'widthHeightDivisor':1,
                'aspectRatios':['1:1','16:9'],'defaultAspectRatio':'1:1',
                'resolutions':['1K','2K'],'defaultResolution':'1K',
                'qualities':['low','medium'],'defaultQuality':'medium'},
            'pricing':{'resolutions':{'1K':{'usd':0.07},'2K':{'usd':0.10}},
                'quality':{'1K':{'low':{'usd':0.05},'medium':{'usd':0.07}},
                    '2K':{'low':{'usd':0.07},'medium':{'usd':0.10}}}}}}]
        self.state=self.call('/api/catalog/refresh',{})
        d={**self.draft('Synthetic 🦊 ratio scene'), 'model':'grok-imagine-image-2-0',
            'settings':{'sizing':'ratio','aspect_ratio':'16:9','resolution':'1K','quality':'medium','format':'png'}}
        self.call('/api/draft',d,'PUT');self.state=self.call('/api/state')
        self.assertEqual(self.state['estimate']['usd'],0.07)
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'saved')
        self.assertEqual(Provider.requests[0][1],{'model':'grok-imagine-image-2-0','prompt':d['prompt'],
            'aspect_ratio':'16:9','resolution':'1K','quality':'medium','format':'png','variants':1,
            'safe_mode':False,'enhance_prompt':False,'return_binary':False,'embed_exif_metadata':False})
        self.assertEqual(a['estimate']['usd'],0.07)
        self.assertEqual(a['results'][0]['width'],1)
        self.stop();self.start()
        reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
        for field,value in d['settings'].items():self.assertEqual(reused['settings'][field],value)
        self.assertEqual(self.call('/api/attempts/'+a['id']),a)
    def test_provider_cannot_echo_credential_in_recipe(self):
        Provider.status=422
        Provider.response={'error':{'message':'invalid offline-test', 'offline-test':'secret name', 'api_key':'offline-test'},'seed':'offline-test'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertNotIn('offline-test',json.dumps(a))
        self.assertNotIn('offline-test',json.dumps(self.call('/api/state')))
        self.assertEqual(a['status'],'rejected')
        self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])

    def test_old_quote_cannot_authorize_changed_draft(self):
        d=self.draft()
        old=self.call('/api/state')['preflight']['draft_revision']
        d['settings']={'width':768,'height':1024}
        self.call('/api/draft',d,'PUT')
        with self.assertRaises(urllib.error.HTTPError) as failure:
            self.call('/api/generate',{'draft_id':d['id'],'draft_revision':old})
        self.assertEqual(failure.exception.code,409)
        self.assertEqual(Provider.requests,[])

    def test_catalog_guidance_claims_and_removed_selection_remain_inspectable(self):
        models=self.call('/api/state')['models']
        chroma=next(m for m in models if m['id']=='chroma')
        self.assertIn('photographic',chroma['guidance']['fit'])
        self.assertEqual(chroma['claims']['uncensored']['state'],'asserted')
        self.assertEqual(chroma['claims']['privacy']['state'],'private')
        self.assertEqual(chroma['claims']['upstream_retention']['state'],'zero_retention_claim')
        self.assertEqual(chroma['claims']['additional_protection']['state'],'unknown')
        self.assertEqual(chroma['claims']['upstream_retention']['policy_source'],'https://docs.venice.ai/overview/privacy')
        last_seen=chroma['claims']['privacy']['fetched_at']
        d=self.draft()
        Provider.catalog=[{'id':'brand-new','model_spec':{'name':'Unfamiliar','offline':False,
            'privacy':'unrecognized','pricing':{'generation':{'usd':0.03}},
            'constraints':{'promptCharacterLimit':10000,'widthHeightDivisor':1}}}]
        self.state=self.call('/api/catalog/refresh',{})
        new=next(m for m in self.state['models'] if m['id']=='brand-new')
        self.assertEqual(new['guidance']['status'],'unknown')
        self.assertEqual(new['claims']['privacy']['state'],'unknown')
        self.assertEqual(new['claims']['uncensored']['state'],'unknown')
        self.assertEqual(new['contract']['sizing'],'unknown')
        old=next(m for m in self.state['models'] if m['id']=='chroma')
        self.assertEqual(old['availability'],'historical')
        self.assertEqual(old['claims']['privacy']['fetched_at'],last_seen)
        self.assertEqual(old['price_fetched_at'],last_seen)
        self.assertEqual(self.state['draft']['model'],'chroma')
        self.assertEqual(self.state['draft']['prompt'],d['prompt'])
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(Provider.requests,[])
        self.state=self.call('/api/catalog/refresh',{})
        self.stop();self.start()
        old=next(m for m in self.state['models'] if m['id']=='chroma')
        self.assertEqual(old['price_fetched_at'],last_seen)

    def test_negative_conflicting_and_unrecognized_claims_stay_distinct(self):
        Provider.catalog=[
            {'id':'chroma','model_spec':{'name':'Chroma','offline':False,'privacy':'anonymized','uncensored':False}},
            {'id':'new-explicit-negative','model_spec':{'name':'New','offline':False,'privacy':'unrecognized-mode','uncensored':False}},
            {'id':'new-unrecognized','model_spec':{'name':'Unknown','offline':False,'uncensored':'sometimes'}},
        ]
        self.state=self.call('/api/catalog/refresh',{})
        models={m['id']:m for m in self.state['models']}
        self.assertEqual(models['chroma']['claims']['uncensored']['state'],'conflicting')
        self.assertEqual(models['chroma']['claims']['privacy']['state'],'anonymized')
        self.assertEqual(models['chroma']['claims']['upstream_retention']['state'],'unknown')
        self.assertEqual(models['new-explicit-negative']['claims']['uncensored']['state'],'not_asserted')
        self.assertEqual(models['new-explicit-negative']['claims']['privacy']['raw'],'unrecognized-mode')
        self.assertEqual(models['new-explicit-negative']['claims']['privacy']['state'],'unknown')
        self.assertEqual(models['new-unrecognized']['claims']['uncensored']['state'],'unknown')
        self.assertEqual(models['new-unrecognized']['claims']['uncensored']['raw'],'sometimes')

    def test_styles_cache_and_full_inventory_survive_refresh_failure_and_restart(self):
        Provider.styles=[f'Synthetic style {i}' for i in range(140)]
        self.state=self.call('/api/catalog/refresh',{})
        self.assertEqual(len(self.state['styles']['data']),140)
        cached=self.state['styles']
        Provider.styles_status=503
        self.state=self.call('/api/catalog/refresh',{})
        self.assertEqual(self.state['styles'],cached)
        self.assertIsNotNone(self.state['styles_error'])

        self.stop();self.start()
        self.assertEqual(self.state['styles'],cached)
        self.assertIsNotNone(self.state['styles_error'])

    def test_documented_advanced_controls_validate_dispatch_and_reuse(self):
        Provider.catalog=[{'id':'venice-sd35','model_spec':{'name':'SD35','offline':False,
            'constraints':{'promptCharacterLimit':1500,'widthHeightDivisor':16,'steps':{'default':25,'max':30}},
            'pricing':{'generation':{'usd':0.01}}}}]
        self.state=self.call('/api/catalog/refresh',{})
        d={**self.draft('Synthetic watercolor'), 'model':'venice-sd35',
           'settings':{'sizing':'pixels','width':768,'height':1024,'format':'webp','negative_prompt':'synthetic text',
               'steps':30,'cfg_scale':3,'style_preset':'Analog Film'}}
        self.call('/api/draft',d,'PUT')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'saved')
        outgoing=Provider.requests[-1][1]
        for key in ['negative_prompt','steps','cfg_scale','style_preset','width','height','format']:
            self.assertEqual(outgoing[key],d['settings'][key])
        reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
        for key,value in d['settings'].items():self.assertEqual(reused['settings'][key],value)
        for setting,value in [('width',767),('steps',31),('cfg_scale',0),('style_preset','Not an advertised style'),('seed',7)]:
            with self.subTest(setting=setting):
                invalid={**reused,'id':str(uuid.uuid4()),'settings':{**reused['settings'],setting:value}}
                self.call('/api/draft',invalid,'PUT')
                with self.assertRaises(urllib.error.HTTPError):self.call('/api/generate',{'draft_id':invalid['id']})
        self.assertEqual(len(Provider.requests),1)

    def test_seed_optimization_and_transparency_preserve_exact_choices(self):
        Provider.catalog=json.loads((ROOT/'docs/research/models-2026-11-14.json').read_text())['models']
        self.state=self.call('/api/catalog/refresh',{})
        cases=[
            ('z-image-turbo',{'sizing':'pixels','width':1024,'height':768,'format':'jpeg','seed':-999999999},{'seed':-999999999}),
            ('qwen-image-3',{'sizing':'ratio','aspect_ratio':'16:9','resolution':'2K','format':'png','optimize_prompt':False},{'disable_prompt_optimization_thinking':True}),
            ('qwen-image-3',{'sizing':'ratio','aspect_ratio':'16:9','resolution':'2K','format':'png','optimize_prompt':True,'negative_prompt':'blurry text'},{'disable_prompt_optimization_thinking':False,'negative_prompt':'blurry text'}),
            ('seedream-v5-5',{'sizing':'ratio','aspect_ratio':'1:1','resolution':'1.5K','format':'webp','transparent_background':True},{'transparent_background':True}),
        ]
        for model,settings,expected in cases:
            with self.subTest(model=model,settings=settings):
                d={**self.draft(),'model':model,'safe_mode':True,'settings':settings}
                self.call('/api/draft',d,'PUT')
                a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
                self.assertEqual(a['status'],'saved')
                for key,value in expected.items():self.assertEqual(Provider.requests[-1][1][key],value)
                self.assertTrue(Provider.requests[-1][1]['safe_mode'])
                self.assertFalse(Provider.requests[-1][1]['enhance_prompt'])
                self.assertEqual(a['requested_seed'],settings.get('seed'))
                self.assertEqual(a['returned_seed'],42)
                self.assertEqual(a['results'][0]['mime'],'image/png')
                reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
                for key,value in settings.items():self.assertEqual(reused['settings'][key],value)

    def test_large_4k_original_is_durable_and_downloadable_after_restart(self):
        row=b'\0'+b'\x17\x80\xb0\xff'*4096
        large=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',4096,4096,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(row*4096,0))+chunk(b'IEND',b'')
        self.assertGreater(len(large),64*1024*1024)
        Provider.response={'images':[base64.b64encode(large).decode()]}
        d=self.draft('Synthetic 4K retention check')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'saved')
        result=a['results'][0]
        self.assertEqual((result['width'],result['height']),(4096,4096))
        self.assertEqual(self.call('/api/images/'+result['id'],raw=True),large)
        self.stop();self.start()
        self.assertEqual(self.call('/api/images/'+result['id'],raw=True),large)
        self.assertEqual(self.call('/api/attempts/'+a['id'])['results'][0]['sha256'],hashlib.sha256(large).hexdigest())

    def test_every_researched_generation_model_has_guidance_and_a_valid_base_request(self):
        Provider.catalog=json.loads((ROOT/'docs/research/models-2026-11-14.json').read_text())['models']
        self.state=self.call('/api/catalog/refresh',{})
        self.assertEqual(len(self.state['models']),48)
        self.assertEqual(sum(m['claims']['uncensored']['state']=='asserted' for m in self.state['models']),14)
        self.assertEqual(sum(m['claims']['uncensored']['state']=='unknown' for m in self.state['models']),34)
        for model in Provider.catalog:
            with self.subTest(model=model['id']):
                metadata=model['model_spec'];constraints=metadata['constraints']
                view=next(m for m in self.state['models'] if m['id']==model['id'])
                self.assertEqual(view['guidance']['status'],'reviewed');self.assertTrue(view['guidance']['sources'])
                settings={'format':'png'}
                if 'aspectRatios' in constraints:
                    settings.update(sizing='ratio',aspect_ratio=constraints['defaultAspectRatio'])
                    if 'resolutions' in constraints:settings['resolution']=constraints['defaultResolution']
                    if 'qualities' in constraints:settings['quality']=constraints['defaultQuality']
                else:settings.update(sizing='pixels',width=1024,height=1024)
                d={**self.draft(),'model':model['id'],'settings':settings}
                self.call('/api/draft',d,'PUT')
                a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
                self.assertEqual(a['status'],'saved');self.assertEqual(a['request']['model'],model['id'])
                if settings['sizing']=='ratio':self.assertNotIn('width',a['request']);self.assertNotIn('height',a['request'])
                else:self.assertEqual(a['request']['width'],1024)
                self.assertEqual(a['control_contract']['reviewed_at'],'2026-11-14')
                self.assertGreaterEqual(a['estimate']['usd'],0)
                self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        self.assertEqual(len(Provider.requests),48)

    def test_long_advertised_unicode_prompt_and_unknown_search_surcharge(self):
        Provider.catalog=json.loads((ROOT/'docs/research/models-2026-11-14.json').read_text())['models']
        self.state=self.call('/api/catalog/refresh',{})
        prompt='🦊'*32768
        settings={'sizing':'ratio','aspect_ratio':'1:1','resolution':'1K','format':'png','seed':0}
        d={**self.draft(prompt),'model':'nano-banana-2','settings':settings}
        self.call('/api/draft',d,'PUT')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(Provider.requests[-1][1]['prompt'],prompt)
        self.assertEqual(Provider.requests[-1][1]['seed'],0)
        self.assertFalse(Provider.requests[-1][1]['enable_web_search'])
        self.assertEqual(a['prompt'],prompt)
        d={**d,'id':str(uuid.uuid4()),'settings':{**settings,'web_search':True}}
        self.call('/api/draft',d,'PUT')
        quote=self.call('/api/state')['estimate']
        self.assertEqual(quote['base_usd'],0.10);self.assertNotIn('usd',quote)
        self.assertIn('surcharge',quote['unavailable_reason'])
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(len(Provider.requests),1)
        d={**d,'id':str(uuid.uuid4()),'model':'venice-sd35'}
        self.call('/api/draft',d,'PUT')
        conflicts=self.call('/api/state')['preflight']['errors']
        self.assertTrue(any('1–1500' in e for e in conflicts));self.assertTrue(any('incompatible' in e for e in conflicts))
        self.assertEqual(self.call('/api/state')['draft']['prompt'],prompt)

    def test_legacy_recipe_reuse_preserves_choices_and_unknown_settings_fail_explicitly(self):
        d=self.draft('Synthetic legacy recipe')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.stop()
        path=self.library/(a['id']+'.recipe.json')
        legacy=json.loads(path.read_text());legacy['recipe_version']=1
        for key in ['model_evidence','control_contract','styles_evidence']:legacy.pop(key)
        path.write_text(json.dumps(legacy))
        self.start()
        reused=self.call('/api/attempts/'+a['id']+'/reuse',{})
        self.assertEqual(reused['prompt'],d['prompt'])
        self.assertEqual(reused['settings']['width'],1024)
        self.assertEqual(reused['settings']['format'],'png')
        self.assertEqual(self.call('/api/attempts/'+a['id'])['request'],a['request'])
        for unknown in ['unrecognized request shape',{**a['request'],'future_setting':'retain me'}]:
            self.stop();legacy['request']=unknown;path.write_text(json.dumps(legacy));self.start()
            before=self.call('/api/state')['draft']
            with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/attempts/'+a['id']+'/reuse',{})
            self.assertIn('substituted',failure.exception.read().decode())
            self.assertEqual(self.call('/api/state')['draft'],before)
            self.assertEqual(self.call('/api/attempts/'+a['id'])['request'],unknown)
        self.assertEqual(len(Provider.requests),1)
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

    def test_insufficient_space_blocks_dispatch_and_retains_the_draft(self):
        self.stop()
        fault=pathlib.Path(self.tmp.name)/'space-fault.so'
        marker=pathlib.Path(self.tmp.name)/'space-low'
        subprocess.run(['cc','-shared','-fPIC',str(ROOT/'tests/space_fault.c'),'-o',str(fault),'-ldl'],check=True)
        self.extra_env={'LD_PRELOAD':str(fault),'VENICE_TEST_SPACE_MARKER':str(marker)}
        self.start();d=self.draft();marker.touch()
        with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/generate',{'draft_id':d['id']})
        self.assertIn('256 MiB',failure.exception.read().decode())
        self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])
        self.assertEqual(Provider.requests,[])
        marker.unlink()
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'saved');self.assertEqual(len(Provider.requests),1)

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

    def test_structured_refusal_and_errors_have_actionable_distinct_summaries(self):
        cases=[
            (422,{'error':'Synthetic provider explanation','retryable':False,'suggested_prompt':'<script>advice only</script>','recommended_model':'other','credits_refunded':True},'refused','policy'),
            (422,{'error':'Invalid size','details':{'_errors':[], 'width':{'_errors':['invalid size']}}},'invalid_settings','settings'),
            (422,{'error':'This prose mentions a content policy but is not a structured verdict'},'rejected','not assumed'),
            (402,{'error':{'code':'UNKNOWN_PAYMENT_SHAPE'}},'payment_error','payment'),
            (402,{'error':'balance','code':'INSUFFICIENT_BALANCE'},'balance_error','balance'),
            (402,{'error':'key limit','code':'API_KEY_USD_SPEND_LIMIT_EXCEEDED'},'key_limit_error','spend limit'),
            (422,{'code':'CONTENT_POLICY_VIOLATION'},'refused','No specific reason'.lower()),
            (422,{'code':'IMAGE_TOO_LARGE'},'invalid_settings','settings'),
            (503,{'unfamiliar':{'detail':'synthetic service error'}},'provider_error','provider'),
            (429,{'error':'slow down','retryable':True},'rate_limited','retry'),
        ]
        for status,body,outcome,word in cases:
            with self.subTest(status=status,body=body):
                Provider.status=status;Provider.response=body;Provider.response_headers={'retry-after':'17','x-request-id':'synthetic-error'}
                d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
                self.assertEqual(a['outcome'],outcome);self.assertIn(word,a['summary'].lower())
                self.assertEqual(a['provider']['body'],body)
                self.assertEqual(self.call('/api/state')['draft']['prompt'],d['prompt'])
                if 'credits_refunded' in body:
                    self.assertEqual(a['billing']['provider_assertions']['credits_refunded'],True)
                    self.assertIn('unverified',a['summary'])
                else:self.assertEqual(a['billing']['refund'],'unknown')
        self.assertEqual(len(Provider.requests),len(cases))

    def test_flagged_original_explained_and_legacy_recipe_read_without_rewrite(self):
        Provider.response_headers={'x-venice-is-blurred':'true','x-venice-is-content-violation':'true'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'saved')
        self.assertEqual(a['outcome'],'flagged')
        for word in ['response-wide','blur','violation','reason','unknown','saved']:
            self.assertIn(word,a['summary'].lower())
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        self.stop()
        path=self.library/(a['id']+'.recipe.json')
        legacy=json.loads(path.read_bytes())
        legacy.update(outcome='received',outcome_summary='Received originals; saving locally.',summary='Originals and recipe saved locally.')
        for key in ['counts','can_retry_local_save']:legacy.pop(key,None)
        for result in legacy['results']:result.pop('available',None)
        path.write_text(json.dumps(legacy));before=path.read_bytes()
        self.start();reopened=self.call('/api/attempts/'+a['id'])
        self.assertEqual(reopened['outcome'],'flagged')
        for word in ['response-wide','blur','violation','reason','unknown','saved']:
            self.assertIn(word,reopened['summary'].lower())
        self.assertEqual(path.read_bytes(),before)
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        self.assertEqual(len(Provider.requests),1)

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

    def test_bounded_unknown_and_plain_text_evidence_preserves_flags_and_unknowns(self):
        for payload in [b'Plain synthetic provider error <script>not code</script>', b'["unknown", {"shape": true}]']:
            Provider.status=502;Provider.raw_body=payload
            d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
            body=a['provider']['body']
            if payload.startswith(b'['):self.assertEqual(body,['unknown',{'shape':True}])
            else:self.assertEqual(body['raw_text'],payload.decode())
            self.assertEqual(a['outcome'],'provider_error');self.assertEqual(a['billing']['refund'],'unknown')
        Provider.status=200;Provider.raw_body=None
        Provider.response={'images':[base64.b64encode(PNG).decode()], 'error':{}, 'unknown_detail':'offline-test'+('x'*60000),'refund':{'assertion':'unknown provider shape'}}
        Provider.response_headers={'x-venice-is-blurred':'true','x-venice-is-content-violation':'true'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertTrue(a['provider']['capture_truncated'])
        self.assertIn('blurred',a['summary']);self.assertIn('content violation',a['summary'])
        self.assertIn('truncated',a['summary']);self.assertIn('unverified',a['summary'])
        self.assertNotIn('offline-test',json.dumps(a));self.assertNotIn('saving locally',a['summary'])
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        self.assertLess(len(json.dumps(a['provider'])),32768)
        self.assertEqual(len(Provider.requests),3)

    def test_no_pixel_policy_inference_and_conflicting_omission_counts_stay_distinct(self):
        black=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',1,1,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b'\0\0\0\0\xff'))+chunk(b'IEND',b'')
        Provider.response={'images':[base64.b64encode(black).decode()]}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['outcome'],'received');self.assertNotIn('violation',a['summary'])
        Provider.response['content_policy_violation_count']=1
        Provider.response_headers={'x-venice-content-violation-count':'2'}
        d=self.draft();d['settings']={'variants':4};self.call('/api/draft',d,'PUT')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['outcome'],'partial');self.assertIsNone(a['counts']['provider_omitted'])
        self.assertTrue(a['counts']['omission_count_conflict']);self.assertIn('conflict',a['summary'])
        self.assertEqual(a['provider']['body']['content_policy_violation_count'],1)
        self.assertEqual(a['provider']['headers']['x-venice-content-violation-count'],'2')
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),black)
        self.assertEqual(len(Provider.requests),2)

    def test_redirects_and_invalid_images_never_trigger_fallback(self):
        Provider.status=302;Provider.response_headers={'Location':f'http://127.0.0.1:{self.provider.server_port}/elsewhere'}
        Provider.response={'error':'inspect this suggestion', 'retryable':True, 'recommended_model':'another-model'}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'failed');self.assertEqual(len(Provider.requests),1)
        Provider.status=200;Provider.response_headers={};Provider.raw_body=None;Provider.response={'images':['aW52YWxpZA==']}
        d=self.draft();a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['status'],'failed');self.assertEqual(a['results'],[]);self.assertEqual(len(Provider.requests),2)
    def test_save_retry_preserves_known_rejection_outcome(self):
        Provider.delay=.3;Provider.status=422;Provider.response={'error':'Invalid settings'}
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

    def test_crash_after_first_variant_exposes_available_original_and_lost_bytes(self):
        self.stop();self.extra_env={'VENICE_STUDIO_TEST_CRASH_AFTER_ORIGINAL':'1'};self.start()
        Provider.response={'images':[base64.b64encode(PNG).decode(),base64.b64encode(PNG).decode()]}
        d=self.draft();d['settings']={'variants':2};self.call('/api/draft',d,'PUT')
        a=self.call('/api/generate',{'draft_id':d['id']})
        self.assertEqual(self.process.wait(timeout=2),86);self.log.close()
        self.extra_env={};self.start();a=self.call('/api/attempts/'+a['id'])
        self.assertEqual(a['status'],'save_failed')
        self.assertEqual([r['available'] for r in a['results']],[True,False])
        self.assertFalse(a['can_retry_local_save'])
        self.assertIn('1 of 2',a['summary']);self.assertIn('missing',a['summary'])
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        with self.assertRaises(urllib.error.HTTPError):self.call('/api/images/'+a['results'][1]['id'],raw=True)
        self.assertEqual(len(Provider.requests),1)

    def test_interrupted_json_retains_complete_originals_without_replay(self):
        encoded=base64.b64encode(PNG).decode()
        Provider.raw_body=('{"id":"synthetic-interrupted","images":["'+encoded+'","incomplete').encode()
        Provider.response_headers={'Content-Length':'10000','x-request-id':'interrupted-with-original'}
        d=self.draft();d['settings']={'variants':2};self.call('/api/draft',d,'PUT')
        a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
        self.assertEqual(a['outcome'],'partial');self.assertEqual(a['status'],'saved')
        self.assertEqual(len(a['results']),1)
        self.assertIn('incomplete',a['summary']);self.assertIn('unknown',a['summary'])
        self.assertTrue(a['provider']['body']['body_incomplete'])
        self.assertNotIn(encoded,json.dumps(a));self.assertNotIn('raw_text',a['provider']['body'])
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
        self.stop();self.start()
        self.assertEqual(self.call('/api/images/'+a['results'][0]['id'],raw=True),PNG)
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
        Provider.response_headers={'Content-Length':str(193*1024*1024)};Provider.raw_body=b''
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
    def test_first_path_estimate_matches_dispatch_pricing(self):
        for pricing,reason in [
            ({'generation':{'usd':0.02}},None),
            ({'generation':{'usd':0.02},'resolutions':{'1K':{'usd':0.08}}},'pricing contract changed'),
            ({'generation':{'usd':'unknown'}},'No usable estimate'),
        ]:
            with self.subTest(pricing=pricing):
                Provider.pricing=pricing
                d=self.draft()
                self.state=self.call('/api/catalog/refresh',{})
                estimate=self.state['estimate']
                requests=len(Provider.requests)
                if reason is None:
                    self.assertEqual(estimate['usd'],0.02)
                    self.assertEqual(estimate['fetched_at'],self.state['catalog']['fetched_at'])
                    a=self.finish(self.call('/api/generate',{'draft_id':d['id']})['id'])
                    self.assertEqual(a['estimate'],estimate)
                    self.assertEqual(len(Provider.requests),requests+1)
                else:
                    self.assertNotIn('usd',estimate)
                    self.assertIn(reason,estimate['unavailable_reason'])
                    with self.assertRaises(urllib.error.HTTPError) as failure:self.call('/api/generate',{'draft_id':d['id']})
                    self.assertEqual(json.loads(failure.exception.read())['error'],estimate['unavailable_reason'])
                    self.assertEqual(len(Provider.requests),requests)
if __name__=='__main__':unittest.main()
