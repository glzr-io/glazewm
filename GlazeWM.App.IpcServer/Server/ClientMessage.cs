using System;

namespace GlazeWM.App.IpcServer.Server
{
  internal sealed record ClientMessage(Guid SessionId, string Message);
}
