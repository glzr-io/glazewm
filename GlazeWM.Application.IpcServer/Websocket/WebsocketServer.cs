using System.Net;
using System.Reactive.Subjects;
using NetCoreServer;

namespace GlazeWM.Application.IpcServer.Websocket
{
  internal sealed class WebsocketServer : WsServer
  {
    /// <summary>
    /// Messages received from all websocket sessions.
    /// </summary>
    public readonly Subject<WebsocketMessage> Messages = new();

    public WebsocketServer(int port) : base(IPAddress.Any, port)
    {
    }

    protected override TcpSession CreateSession()
    {
      return new WebsocketSession(this);
    }

    protected override void Dispose(bool disposingManagedResources)
    {
      if (disposingManagedResources)
        Messages.Dispose();

      base.Dispose(disposingManagedResources);
    }
  }
}
