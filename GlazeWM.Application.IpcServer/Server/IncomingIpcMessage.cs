using System;

namespace GlazeWM.Application.IpcServer.Server
{
  internal record IncomingIpcMessage(Guid SessionId, string Text);
}
