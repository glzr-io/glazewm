using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Interprocess.Modules;
using GlazeWM.Interprocess.Websocket;
using Microsoft.Extensions.Logging;
using Qmmands;

namespace GlazeWM.Interprocess
{
  public sealed class InterprocessService : IDisposable
  {
    private readonly ILogger<InterprocessService> _logger;

    private readonly IServiceProvider _serviceProvider;

    private readonly Bus _bus;

    private readonly ContainerService _containerService;

    private readonly WindowService _windowService;

    private readonly WorkspaceService _workspaceService;

    private readonly WebsocketServer _server = new();

    private readonly CommandService _commandService = new();

    public InterprocessService(
      ILogger<InterprocessService> logger,
      IServiceProvider serviceProvider,
      Bus bus,
      ContainerService containerService,
      WindowService windowService,
      WorkspaceService workspaceService
    )
    {
      _logger = logger;
      _serviceProvider = serviceProvider;
      _bus = bus;
      _containerService = containerService;
      _windowService = windowService;
      _workspaceService = workspaceService;

      Initialize();
    }

    public void Dispose()
    {
      if (!_server.IsDisposed)
      {
        Stop();

        _server.Dispose();
      }
    }

    public void Start()
    {
      if (_server.IsStarted)
      {
        _logger.LogDebug("The server has already been started");

        return;
      }

      _server.Start();

      _logger.LogDebug("Started listening on port {Port}", _server.Port);
    }

    public void Stop()
    {
      if (!_server.IsStarted)
      {
        _logger.LogDebug("The server has not been started yet");

        return;
      }

      _server.Stop();

      _logger.LogDebug("Stopped listening on port {Port}", _server.Port);
    }

    private void Initialize()
    {
      _commandService.AddModule<WorkspaceModule>();

      _server.MessageReceived.Subscribe(HandleCommand);
    }

    private async void HandleCommand(WebsocketMessage message)
    {
      var context = new InterprocessContext(_serviceProvider, message, _server);

      await _commandService.ExecuteAsync(message.Text, context);
    }
  }
}
