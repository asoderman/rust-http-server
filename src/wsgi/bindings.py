
class Application:

    def __init__(self):
        self.headers = []

    def start_response(self, status, response_headers, exc_info=None):
        self.headers = [status, response_headers]

    def finish_response(self):
        pass

    @classmethod
    def call_callable(cls, env, application):

        app_state = cls()

        body = application(env, app_state.start_response)

        status, headers = app_state.headers
        result = 'HTTP/1.1 {}\r\n'.format(status)
        for h in headers:
            result += '{0}: {1}\r\n'.format(*h)
        result += '\r\n'

        for data in body:
            result += data.decode()

        return result
        ## Implement function that calls the wsgi app and returns data
        ## back to rust
