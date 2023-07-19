using System.Text;
using NetCoreServer;

namespace GlazeWM.Interprocess.Websocket
{
  internal sealed class WebsocketSession : WsSession
  {
    public WebsocketSession(WsServer server) : base(server)
    {
    }

    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      var text = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
      var message = new WebsocketMessage(Id, text);

      if (Server is WebsocketServer server)
        server.MessageReceived.OnNext(message);
    }
  }
}
