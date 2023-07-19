using System.Text;
using NetCoreServer;

namespace GlazeWM.Interprocess.Websocket
{
  internal sealed class WebsocketSession : WsSession
  {
    public WebsocketSession(WebsocketServer server) : base(server)
    {
    }

    /// <summary>
    /// Emit received text buffers to `Messages` subject.
    /// </summary>
    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      var text = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
      server.Messages.OnNext(new WebsocketMessage(Id, text););
    }
  }
}
