using System.Net;
using System.Reactive.Subjects;
using NetCoreServer;

namespace GlazeWM.Application.IpcServer.Server
{
  internal sealed class IpcServer : WsServer
  {
    /// <summary>
    /// Messages received from all websocket sessions.
    /// </summary>
    public readonly Subject<IncomingIpcMessage> Messages = new();

    public IpcServer(int port) : base(IPAddress.Any, port)
    {
    }

    protected override TcpSession CreateSession()
    {
      return new IpcSession(this);
    }

    protected override void Dispose(bool disposingManagedResources)
    {
      if (disposingManagedResources)
        Messages.Dispose();

      base.Dispose(disposingManagedResources);
    }
  }
}
