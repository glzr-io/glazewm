using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Logging;

namespace GlazeWM.App.IpcServer
{
  public sealed class IpcServerStartup
  {
    private readonly ILogger<IpcServerStartup> _logger;
    private readonly IpcMessageHandler _ipcMessageHandler;

    public IpcServerStartup(
      ILogger<IpcServerStartup> logger,
      IpcMessageHandler ipcMessageHandler)
    {
      _logger = logger;
      _ipcMessageHandler = ipcMessageHandler;
    }

    /// <summary>
    /// Start the IPC server on specified port.
    /// </summary>
    public void Run(int port)
    {
      _logger.LogDebug("Starting IPC server on port {Port}.", port);

      _ipcMessageHandler.Init();

      var builder = WebApplication.CreateBuilder();
      builder.Logging.ClearProviders();
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
