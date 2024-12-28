export const idlFactory = ({ IDL }) => {
  const Classification = IDL.Record({
    'label' : IDL.Text,
    'score' : IDL.Float32,
  });
  const ClassificationError = IDL.Record({ 'message' : IDL.Text });
  const ClassificationResult = IDL.Variant({
    'Ok' : IDL.Vec(Classification),
    'Err' : ClassificationError,
  });
  const HeaderField = IDL.Tuple(IDL.Text, IDL.Text);
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
    'status_code' : IDL.Nat16,
  });
  return IDL.Service({
    'classify' : IDL.Func([IDL.Vec(IDL.Nat8)], [ClassificationResult], []),
    'classify_query' : IDL.Func(
        [IDL.Vec(IDL.Nat8)],
        [ClassificationResult],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'llm' : IDL.Func([IDL.Text], [IDL.Text], []),
    'run' : IDL.Func([], [ClassificationResult], ['query']),
    'send_http_post_request' : IDL.Func([IDL.Text], [IDL.Text], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
