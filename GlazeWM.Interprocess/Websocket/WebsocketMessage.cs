using System;

namespace GlazeWM.Interprocess.Websocket
{
  internal sealed class WebsocketMessage
  {
    public Guid SessionId { get; }

    public string Text { get; }

    public WebsocketMessage(Guid sessionId, string text)
    {
      SessionId = sessionId;
      Text = text;
    }
  }
}
