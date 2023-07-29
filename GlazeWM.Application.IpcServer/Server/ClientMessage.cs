using System;

namespace GlazeWM.Application.IpcServer.Server
{
  internal record ClientMessage(Guid SessionId, string Message);
}
