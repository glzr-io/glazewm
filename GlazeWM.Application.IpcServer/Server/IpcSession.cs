using System.Text;
using NetCoreServer;

namespace GlazeWM.Application.IpcServer.Server
{
  internal sealed class IpcSession : WsSession
  {
    public IpcSession(IpcServer server) : base(server)
    {
    }

    /// <summary>
    /// Emit received text buffers to `Messages` subject of the server.
    /// </summary>
    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      var text = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);

      if (Server is IpcServer server)
        server.Messages.OnNext(new IncomingIpcMessage(Id, text));
    }
  }
}
