using System;

namespace GlazeWM.App.IpcServer.Server
{
  internal record ClientMessage(Guid SessionId, string Message);
}
