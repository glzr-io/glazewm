using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;

namespace GlazeWM.App.IpcServer
{
  public sealed class IpcServerStartup
  {
    private readonly IpcMessageHandler _ipcMessageHandler;

    public IpcServerStartup(IpcMessageHandler ipcMessageHandler)
    {
      _ipcMessageHandler = ipcMessageHandler;
    }

    /// <summary>
    /// Start the IPC server on specified port.
    /// </summary>
    public void Run(int port)
    {
      var builder = WebApplication.CreateBuilder();
      builder.WebHost.UseUrls($"http://localhost:{port}");

      var app = builder.Build();
      app.UseWebSockets();

      app.Use(async (context, next) =>
      {
        if (!context.WebSockets.IsWebSocketRequest)
          await next();

        using var ws = await context.WebSockets.AcceptWebSocketAsync();
        await _ipcMessageHandler.Handle(ws);
      });

      app.Run();
    }
  }
}
