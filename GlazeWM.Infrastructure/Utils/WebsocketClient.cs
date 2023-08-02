using System;
using System.Reactive.Subjects;
using System.Text;
using NetCoreServer;

namespace GlazeWM.Infrastructure.Utils
{
  public class WebsocketClient : WsClient
  {
    /// <summary>
    /// Messages received from websocket server.
    /// </summary>
    public readonly ReplaySubject<string> Messages = new();

    public WebsocketClient(int port) : base("127.0.0.1", port) { }

    public WebsocketClient(string address, int port) : base(address, port) { }

    public override void OnWsConnecting(HttpRequest request)
    {
      request.SetBegin("GET", "/")
        .SetHeader("Host", "localhost")
        .SetHeader("Origin", "http://localhost")
        .SetHeader("Upgrade", "websocket")
        .SetHeader("Connection", "Upgrade")
        .SetHeader("Sec-WebSocket-Key", Convert.ToBase64String(WsNonce))
        .SetHeader("Sec-WebSocket-Protocol", "chat, superchat")
        .SetHeader("Sec-WebSocket-Version", "13")
        .SetBody();
    }

    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      var message = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
      Messages.OnNext(message);
    }
  }
}
