using System;
using GlazeWM.Interprocess.Websocket;
using Qmmands;

namespace GlazeWM.Interprocess
{
  internal sealed class InterprocessContext : CommandContext
  {
    public WebsocketMessage Message { get; }

    public WebsocketServer Server { get; }

    public InterprocessContext(
      IServiceProvider serviceProvider,
      WebsocketMessage message,
      WebsocketServer server) : base(serviceProvider)
    {
      Message = message;
      Server = server;
    }
  }
}
