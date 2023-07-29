namespace GlazeWM.Application.IpcServer.Server
{
  internal record ServerMessage<T>(ServerMessagePayloadType PayloadType, T Payload);
}
