using System;
using System.Net.WebSockets;

namespace GlazeWM.App.IpcServer
{
  public sealed record EventSubscription(Guid SubscriptionId, WebSocket WebSocket);
}
