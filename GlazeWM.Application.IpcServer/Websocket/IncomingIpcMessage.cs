using System;

namespace GlazeWM.Application.IpcServer.Websocket
{
  internal sealed class IncomingIpcMessage
  {
    public Guid SessionId { get; }
    public string Text { get; }

    public IncomingIpcMessage(Guid sessionId, string text)
    {
      SessionId = sessionId;
      Text = text;
    }
  }
}
