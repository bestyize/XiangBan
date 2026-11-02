import json
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import worker as w


class EditorialTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.db = w.connect(self.root)
        self.addCleanup(self.db.close)

    def test_source_scope_and_private_network_rejection(self):
        with self.assertRaises(w.Reject):
            w.source_url('https://evil.example/a', ['docs.example'])
        with patch('worker.socket.getaddrinfo', return_value=[(2, 1, 6, '', ('127.0.0.1', 443))]):
            with self.assertRaises(w.Reject):
                w.source_url('https://docs.example/a', ['docs.example'])
        for url in ['http://docs.example', 'https://a:b@docs.example', 'https://docs.example:444/a']:
            with self.assertRaises(w.Reject):
                w.canonical(url)

    def test_same_source_is_one_job_across_agents(self):
        w.enqueue(self.db, 'learn-a', 'https://docs.example/a/#part', 'first')
        w.enqueue(self.db, 'learn-b', 'https://docs.example/a', 'second')
        self.assertEqual(self.db.execute('SELECT count(*) FROM jobs').fetchone()[0], 1)
        self.assertEqual(self.db.execute('SELECT url FROM jobs').fetchone()[0], 'https://docs.example/a/')

    def test_fetch_preserves_source_trailing_slash(self):
        with patch('worker.socket.getaddrinfo', return_value=[(2, 1, 6, '', ('1.1.1.1', 443))]):
            self.assertEqual(w.source_url('https://docs.example/feed/', ['docs.example']), 'https://docs.example/feed/')

    def test_feed_retains_source_date_and_skips_stale_and_future_items(self):
        def item(name, days):
            date = (w.dt.datetime.now(w.UTC) - w.dt.timedelta(days=days)).strftime('%a, %d %b %Y %H:%M:%S %z')
            return f'<item><title>{name}</title><link>https://docs.example/{name}</link><pubDate>{date}</pubDate></item>'
        xml = '<rss><channel>' + item('recent', 1) + item('stale', 90) + item('future', -3) + '</channel></rss>'
        config = {'agents': [{'id': 'frontier', 'enabled': True, 'hosts': ['docs.example'], 'sources': [{'type': 'feed', 'url': 'https://docs.example/feed/', 'max_age_days': 30, 'require_date': True}]}]}
        with patch('worker.fetch_source', return_value=xml), patch('worker.source_url', side_effect=lambda url, hosts: url):
            w.discover(self.db, config)
        rows = self.db.execute('SELECT source_title,source_published_at FROM jobs').fetchall()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['source_title'], 'recent')
        self.assertIsNotNone(rows[0]['source_published_at'])

    def test_disjoint_model_objects_merge_but_ambiguous_or_prose_rejected(self):
        self.assertEqual(w.model_json('{"title":"a"}\n{"body":"b"}'), {'title': 'a', 'body': 'b'})
        for text in ['{"approved":true}{"approved":false}', '{"approved":false,"approved":true}', 'APPROVED {"approved":true}', '{"approved":true} explanation']:
            with self.assertRaises(ValueError):
                w.model_json(text)

    def test_ambiguous_import_recovers_existing_published_draft(self):
        w.enqueue(self.db, 'learn-a', 'https://docs.example/a', 'first')
        job = self.db.execute('SELECT * FROM jobs').fetchone()
        worker = object.__new__(w.Worker)
        worker.root = self.root
        with patch.object(worker, 'existing_draft', return_value={'id': 'draft', 'topic_id': 123}), patch.object(worker, 'verify') as verify, patch('worker.fetch_source') as source:
            worker.process(job, {'id': 'learn-a', 'destination': 'topic'})
            verify.assert_called_once_with(job['id'], 123)
            source.assert_not_called()

    def test_ambiguous_import_recovers_unpublished_draft_without_model_cost(self):
        w.enqueue(self.db, 'learn-a', 'https://docs.example/a', 'first')
        job = self.db.execute('SELECT * FROM jobs').fetchone()
        worker = object.__new__(w.Worker)
        worker.root, worker.db = self.root, self.db
        with patch.object(worker, 'existing_draft', return_value={'id': 'draft', 'topic_id': None}), patch.object(worker, 'publish') as publish, patch('worker.fetch_source') as source:
            worker.process(job, {'id': 'learn-a', 'destination': 'topic'})
            publish.assert_called_once_with(job['id'], 'draft')
            source.assert_not_called()

    def test_draft_recovery_uses_exact_id_and_rejects_unfiltered_old_servers(self):
        worker=object.__new__(w.Worker);worker.forum='https://forum.example';worker.editor_token='private'
        with patch('worker.api',return_value={'items':[{'external_id':'ed-123','id':'draft'}]}) as request:
            self.assertEqual(worker.existing_draft('ed-123')['id'],'draft')
            self.assertEqual(request.call_args.args[1],'/api/editor/drafts?external_id=ed-123')
        with patch('worker.api',return_value={'items':[{'external_id':'different'}]}):
            with self.assertRaises(w.Stop):worker.existing_draft('ed-123')

    def test_discover_comparison_accepts_equivalent_timestamp_only(self):
        a={'title':'标题','source_published_at':'2026-10-22T07:51:31+00:00','body':'正文'}
        self.assertTrue(w.discover_matches(dict(a,source_published_at='2026-10-22T07:51:31Z'),a))
        self.assertFalse(w.discover_matches(dict(a,source_published_at='2026-10-23T07:51:31Z'),a))
        self.assertFalse(w.discover_matches(dict(a,body='被他人修改'),a))

    def test_transient_reads_retry_but_paid_posts_do_not(self):
        error=w.urllib.error.HTTPError('https://forum.example',502,'unavailable',{},None)
        from unittest.mock import MagicMock
        response=MagicMock();response.__enter__.return_value.read.return_value=b'{"ok":true}'
        with patch.object(w.HTTP,'open',side_effect=[error,response]) as request,patch('worker.time.sleep'):
            self.assertTrue(w.api('https://forum.example','/api/categories')['ok'])
            self.assertEqual(request.call_count,2)
        with patch.object(w.HTTP,'open',side_effect=error) as request,patch('worker.time.sleep'):
            with self.assertRaises(w.HTTPFailure):w.api('https://ai.example','/v1/images/generations',data={'prompt':'example'})
            self.assertEqual(request.call_count,1)

    def test_budget_blocks_paid_call_before_reservation(self):
        worker = object.__new__(w.Worker)
        worker.db, worker.gateway = self.db, 'https://ai.example'
        worker.config = {'reserve_points': 100, 'daily_spend_points': 1000, 'daily_call_limit': 10}
        with patch.object(worker, 'ai_token', return_value='private'), patch('worker.api', return_value={'total': 50}):
            with self.assertRaises(w.Stop):
                worker.model('job', 'chat/completions', {})
        self.assertEqual(self.db.execute('SELECT count(*) FROM calls').fetchone()[0], 0)

    def test_parallel_calls_cannot_exceed_the_reserved_call_limit(self):
        def request(base,path,*args,**kwargs):return {'total':1000} if path=='/api/ai/balance' else {}
        def invoke(i):
            worker=object.__new__(w.Worker);worker.root=self.root;worker.db=w.connect(self.root);worker.gateway='https://ai.example'
            worker.config={'reserve_points':0,'daily_spend_points':10000,'daily_call_limit':2}
            try:
                with patch.object(worker,'ai_token',return_value='private'):worker.model(str(i),'chat/completions',{})
                return True
            except w.Stop:return False
            finally:worker.db.close()
        with patch('worker.api',side_effect=request), w.concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            results=list(pool.map(invoke,range(3)))
        self.assertEqual(sum(results),2)
        self.assertEqual(self.db.execute('SELECT count(*) FROM calls').fetchone()[0],2)

    def test_duplicate_gate_ignores_changed_titles(self):
        worker=object.__new__(w.Worker);worker.db=self.db
        original='可核对的具体步骤和实际功能限制。'*100
        self.db.execute('INSERT INTO article_signatures(job_id,signature) VALUES(?,?)',('old',json.dumps(w.article_signature(original))));self.db.commit()
        with self.assertRaises(w.Reject):worker.check_duplicate('new',original)
        worker.check_duplicate('different','一份完全不同的电影档案与作品背景说明。'*100)

    def test_index_follows_observed_pagination_and_preserves_source_scope(self):
        pages = {
            'https://docs.example/guides/': '<a href="one">One</a><a href="?page=2">Next</a><a href="https://evil.example/guides/injected">bad</a>',
            'https://docs.example/guides/?page=2': '<a href="two">Two</a><a href="?page=3">Next</a>',
        }
        with patch('worker.cached_source', side_effect=lambda db,url,*args: pages[url]) as fetch:
            w.collect_index(self.db, {}, {'id':'learn-a','hosts':['docs.example']}, {'url':'https://docs.example/guides/','prefixes':['/guides/'],'max_pages':2})
        self.assertEqual(fetch.call_count,2)
        self.assertEqual({r['url'] for r in self.db.execute('SELECT url FROM jobs')}, {'https://docs.example/guides/one','https://docs.example/guides/two'})

    def test_notebook_evidence_excludes_embedded_execution_outputs(self):
        raw=json.dumps({'cells':[{'cell_type':'markdown','source':['Verified documentation. '*50]}, {'cell_type':'code','source':['print(1)'],'outputs':[{'text':['NOT_SOURCE_SECRET']}]}]})
        text,_=w.plain_page(raw)
        self.assertIn('Verified documentation.',text)
        self.assertIn('print(1)',text)
        self.assertNotIn('NOT_SOURCE_SECRET',text)

    def test_content_gate_rejects_short_copy_and_unapproved_assets(self):
        a = {'title': '正文标题', 'summary': '摘要', 'body': '实' * 1300, 'image_prompt': 'illustration'}
        w.validate_article(a)
        for body in ['too short', '实' * 1300 + '\n![a](https://evil.example/a.png)', '实' * 1300 + '{{image:1}}']:
            with self.assertRaises(w.Reject):
                w.validate_article(dict(a, body=body))

    def test_artifacts_private_and_main_article_text_preferred(self):
        p = self.root / 'article.json'
        w.atomic(p, {'title': 'example'})
        self.assertEqual(p.stat().st_mode & 0o777, 0o600)
        text, dates = w.plain_page('<nav>ignore</nav><article><h1>Title</h1><p>' + 'fact ' * 200 + '</p></article><aside>other</aside>')
        self.assertNotIn('ignore', text)
        self.assertNotIn('other', text)

    def test_unsupported_destination_cannot_be_enabled(self):
        p = self.root / 'agents.json'
        p.write_text(json.dumps({'agents': [{'id': 'create-game', 'enabled': True, 'destination': 'pack'}]}))
        with self.assertRaises(w.Stop):
            w.load_config(p)

    def test_publication_checks_remote_content_and_uses_revision_precondition(self):
        worker = object.__new__(w.Worker)
        worker.root, worker.forum, worker.editor_token = self.root, 'https://forum.example', 'private'
        article = {'title': '标题', 'body': '正文'}
        dest = self.root / 'articles' / 'job'
        w.atomic(dest / 'article.json', article)
        w.atomic(dest / 'review.json', {'approved': True, 'issues': [], 'article_sha256': w.hashlib.sha256(json.dumps(article, sort_keys=True, ensure_ascii=False).encode()).hexdigest()})
        body = '正文\n![封面](images/cover.png)'
        images = [{'path': 'images/cover.png', 'data_base64': 'aW1hZ2U='}]
        w.atomic(dest / 'import.json', {'title': '标题', 'body': body, 'author_uid': 'author', 'category': 'tech-llm', 'tags': ['TEST'], 'images': images})
        remote = {'draft': {'title': '标题', 'author_uid': 'author', 'tags': ['test'], 'updated_at': '2026-11-02T00:00:00Z'}, 'body': body.replace('](images/cover.png)', '](<images/cover.png>)'), 'category': 'tech-llm', 'images': images}
        with patch('worker.api', side_effect=[remote, {'id': 123}]) as api, patch.object(worker, 'verify'):
            worker.publish('job', 'draft')
            self.assertEqual(api.call_args.kwargs['if_match'], remote['draft']['updated_at'])
        remote['body'] += '管理员修改'
        with patch('worker.api', return_value=remote) as api:
            with self.assertRaises(w.Reject):
                worker.publish('job', 'draft')
            self.assertEqual(api.call_count, 1)

    def test_frontier_recovery_verifies_existing_publication_without_regenerating(self):
        w.enqueue(self.db, 'discover-frontier', 'https://docs.example/news', 'news')
        job = self.db.execute('SELECT * FROM jobs').fetchone()
        worker = object.__new__(w.Worker)
        worker.root = self.root
        with patch.object(worker, 'frontier_draft', return_value={'status': 'published'}), patch.object(worker, 'verify_frontier') as verify, patch('worker.fetch_source') as source:
            worker.process(job, {'id': 'discover-frontier', 'destination': 'frontier'})
            verify.assert_called_once_with(job['id'],'frontier')
            source.assert_not_called()

    def test_bounded_runs_do_not_starve_later_agents(self):
        agents = [{'id': f'agent-{i}', 'name': f'Agent {i}', 'destination': 'topic', 'enabled': True} for i in range(7)]
        for a in agents:
            w.enqueue(self.db, a['id'], 'https://docs.example/' + a['id'], a['name'])
        processed = []
        def publish(job, agent):
            processed.append(agent['id'])
            self.db.execute("UPDATE jobs SET status='published' WHERE id=?", (job['id'],))
            self.db.commit()
        with patch('worker.Worker') as cls, patch('worker.discover'), contextlib.redirect_stdout(io.StringIO()):
            cls.return_value.process.side_effect = publish
            for _ in range(3):
                w.run(self.root, {'agents': agents}, self.db, 3)
        self.assertEqual(set(processed), {a['id'] for a in agents})
        self.assertEqual(len(processed), 7)


if __name__ == '__main__':
    unittest.main()
