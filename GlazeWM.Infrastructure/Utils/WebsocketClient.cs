using System;
using System.Net.Sockets;
using System.Reactive.Subjects;
using System.Text;
using System.Threading;
using NetCoreServer;

namespace GlazeWM.Infrastructure.Utils
{
  public class WebsocketClient : WsClient
  {
    /// <summary>
    /// Messages received from websocket server.
    /// </summary>
    public readonly Subject<string> Messages = new();

    public WebsocketClient(int port) : base("127.0.0.1", port) { }

    public WebsocketClient(string address, int port) : base(address, port) { }

    public void DisconnectAndStop()
    {
      _stop = true;
      CloseAsync(1000);
      while (IsConnected)
        Thread.Yield();
    }

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
    }

    public override void OnWsDisconnected()
    {
      Console.WriteLine($"Chat WebSocket client disconnected a session with Id {Id}");
    }

    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      Console.WriteLine($"Incoming: {Encoding.UTF8.GetString(buffer, (int)offset, (int)size)}");
      var message = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
      Messages.OnNext(message);
    }

    protected override void OnDisconnected()
    {
      base.OnDisconnected();

      Console.WriteLine($"Chat WebSocket client disconnected a session with Id {Id}");

      // Wait for a while...
      Thread.Sleep(1000);

      // Try to connect again
      if (!_stop)
        ConnectAsync();
    }

    protected override void OnError(SocketError error)
    {
      Console.WriteLine($"Chat WebSocket client caught an error with code {error}");
    }

    private bool _stop;
  }
}
