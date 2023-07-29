namespace GlazeWM.Application.IpcServer.Server
{
  internal record OutgoingIpcMessage<T>(IpcPayloadType PayloadType, T Payload);
}
