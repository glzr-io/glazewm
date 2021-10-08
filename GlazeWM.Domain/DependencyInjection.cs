using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.CommandHandlers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.CommandHandlers;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.CommandHandlers;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.CommandHandlers;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Windows.EventHandlers;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.CommandHandlers;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddDomainServices(this IServiceCollection services)
    {
      services.AddSingleton<ContainerService>();
      services.AddSingleton<MonitorService>();
      services.AddSingleton<CommandParsingService>();
      services.AddSingleton<UserConfigService>();
      services.AddSingleton<WindowService>();
      services.AddSingleton<WorkspaceService>();

      services.AddTransient<ICommandHandler<AttachContainerCommand>, AttachContainerHandler>();
      services.AddTransient<ICommandHandler<ChangeContainerLayoutCommand>, ChangeContainerLayoutHandler>();
      services.AddTransient<ICommandHandler<ChangeFocusedContainerLayoutCommand>, ChangeFocusedContainerLayoutHandler>();
      services.AddTransient<ICommandHandler<DetachContainerCommand>, DetachContainerHandler>();
      services.AddTransient<ICommandHandler<RedrawContainersCommand>, RedrawContainersHandler>();
      services.AddTransient<ICommandHandler<ReplaceContainerCommand>, ReplaceContainerHandler>();
      services.AddTransient<ICommandHandler<SetFocusedDescendantCommand>, SetFocusedDescendantHandler>();
      services.AddTransient<ICommandHandler<SwapContainersCommand>, SwapContainersHandler>();
      services.AddTransient<ICommandHandler<AddMonitorCommand>, AddMonitorHandler>();
      services.AddTransient<ICommandHandler<EvaluateUserConfigCommand>, EvaluateUserConfigHandler>();
      services.AddTransient<ICommandHandler<RegisterKeybindingsCommand>, RegisterKeybindingsHandler>();
      services.AddTransient<ICommandHandler<AddWindowCommand>, AddWindowHandler>();
      services.AddTransient<ICommandHandler<CloseFocusedWindowCommand>, CloseFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<FocusInDirectionCommand>, FocusInDirectionHandler>();
      services.AddTransient<ICommandHandler<FocusWindowCommand>, FocusWindowHandler>();
      services.AddTransient<ICommandHandler<MoveFocusedWindowCommand>, MoveFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<RemoveWindowCommand>, RemoveWindowHandler>();
      services.AddTransient<ICommandHandler<ResizeFocusedWindowCommand>, ResizeFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<ToggleFloatingCommand>, ToggleFloatingHandler>();
      services.AddTransient<ICommandHandler<AttachWorkspaceToMonitorCommand>, AttachWorkspaceToMonitorHandler>();
      services.AddTransient<ICommandHandler<CreateWorkspaceCommand>, CreateWorkspaceHandler>();
      services.AddTransient<ICommandHandler<DetachWorkspaceFromMonitorCommand>, DetachWorkspaceFromMonitorHandler>();
      services.AddTransient<ICommandHandler<DisplayWorkspaceCommand>, DisplayWorkspaceHandler>();
      services.AddTransient<ICommandHandler<FocusWorkspaceCommand>, FocusWorkspaceHandler>();
      services.AddTransient<ICommandHandler<MoveFocusedWindowToWorkspaceCommand>, MoveFocusedWindowToWorkspaceHandler>();

      services.AddTransient<IEventHandler<WindowDestroyedEvent>, WindowDestroyedHandler>();
      services.AddTransient<IEventHandler<WindowFocusedEvent>, WindowFocusedHandler>();
      services.AddTransient<IEventHandler<WindowHiddenEvent>, WindowHiddenHandler>();
      services.AddTransient<IEventHandler<WindowMinimizedEvent>, WindowMinimizedHandler>();
      services.AddTransient<IEventHandler<WindowMinimizeEndedEvent>, WindowMinimizeEndedHandler>();
      services.AddTransient<IEventHandler<WindowShownEvent>, WindowShownHandler>();

      return services;
    }
  }
}
