import importlib.util
import pathlib
import tempfile
import time
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('auth_store', pathlib.Path(__file__).with_name('auth_store.py'))
store = importlib.util.module_from_spec(spec); spec.loader.exec_module(store)

class SessionTests(unittest.TestCase):
    def test_expired_session_rotates_and_reuses_replacement(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            with patch.object(store,'ROOT',root), patch.object(store,'SESSION',root/'session.json'):
                store.save({'access_token':'old','refresh_token':'old-refresh','expires_in':0})
                with patch.object(store,'request',return_value={'access_token':'new','refresh_token':'new-refresh','expires_in':3600}) as request:
                    self.assertEqual(store.token(),'new')
                    self.assertEqual(store.token(),'new')
                    request.assert_called_once_with('/api/auth/refresh',{'refresh_token':'old-refresh','client_id':'xiaoban-frontier'})
                self.assertEqual((root/'session.json').stat().st_mode & 0o777,0o600)

if __name__ == '__main__': unittest.main()
