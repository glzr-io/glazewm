using System;
using System.Reactive.Subjects;
using System.Text;
using NetCoreServer;

namespace GlazeWM.Infrastructure.Utils
{
  public class WebsocketClient : WsClient
  {
    /// <summary>
    /// Connection event to websocket server.
    /// </summary>
    public readonly Subject<bool> Connected = new();

    /// <summary>
    /// Messages received from websocket server.
    /// </summary>
    public readonly Subject<string> Messages = new();

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

    public override void OnWsConnected(HttpResponse response)
    {
      Console.WriteLine($"Chat WebSocket client connected a new session with Id {Id}");
      Connected.OnNext(true);
    }

    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      Console.WriteLine($"Incoming: {Encoding.UTF8.GetString(buffer, (int)offset, (int)size)}");
      var message = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
      Messages.OnNext(message);
    }
  }
}
