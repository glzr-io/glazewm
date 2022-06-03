using GlazeWM.Domain.Common.CommandHandlers;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.CommandHandlers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.CommandHandlers;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.EventHandlers;
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

      services.AddSingleton<ICommandHandler<ExitApplicationCommand>, ExitApplicationHandler>();
      services.AddSingleton<ICommandHandler<AttachAndResizeContainerCommand>, AttachAndResizeContainerHandler>();
      services.AddSingleton<ICommandHandler<AttachContainerCommand>, AttachContainerHandler>();
      services.AddSingleton<ICommandHandler<ChangeContainerLayoutCommand>, ChangeContainerLayoutHandler>();
      services.AddSingleton<ICommandHandler<ChangeFocusedContainerLayoutCommand>, ChangeFocusedContainerLayoutHandler>();
      services.AddSingleton<ICommandHandler<DetachAndResizeContainerCommand>, DetachAndResizeContainerHandler>();
      services.AddSingleton<ICommandHandler<DetachContainerCommand>, DetachContainerHandler>();
      services.AddSingleton<ICommandHandler<FlattenSplitContainerCommand>, FlattenSplitContainerHandler>();
      services.AddSingleton<ICommandHandler<FocusInDirectionCommand>, FocusInDirectionHandler>();
      services.AddSingleton<ICommandHandler<MoveContainerWithinTreeCommand>, MoveContainerWithinTreeHandler>();
      services.AddSingleton<ICommandHandler<RedrawContainersCommand>, RedrawContainersHandler>();
      services.AddSingleton<ICommandHandler<ReplaceContainerCommand>, ReplaceContainerHandler>();
      services.AddSingleton<ICommandHandler<SetFocusedDescendantCommand>, SetFocusedDescendantHandler>();
      services.AddSingleton<ICommandHandler<ToggleFocusModeCommand>, ToggleFocusModeHandler>();
      services.AddSingleton<ICommandHandler<AddMonitorCommand>, AddMonitorHandler>();
      services.AddSingleton<ICommandHandler<RefreshMonitorStateCommand>, RefreshMonitorStateHandler>();
      services.AddSingleton<ICommandHandler<RemoveMonitorCommand>, RemoveMonitorHandler>();
      services.AddSingleton<ICommandHandler<EvaluateUserConfigCommand>, EvaluateUserConfigHandler>();
      services.AddSingleton<ICommandHandler<RegisterKeybindingsCommand>, RegisterKeybindingsHandler>();
      services.AddSingleton<ICommandHandler<AddWindowCommand>, AddWindowHandler>();
      services.AddSingleton<ICommandHandler<CloseWindowCommand>, CloseWindowHandler>();
      services.AddSingleton<ICommandHandler<FocusWindowCommand>, FocusWindowHandler>();
      services.AddSingleton<ICommandHandler<MoveWindowCommand>, MoveWindowHandler>();
      services.AddSingleton<ICommandHandler<RemoveWindowCommand>, RemoveWindowHandler>();
      services.AddSingleton<ICommandHandler<ResizeFocusedWindowCommand>, ResizeFocusedWindowHandler>();
      services.AddSingleton<ICommandHandler<ResizeFocusedWindowBordersCommand>, ResizeFocusedWindowBordersHandler>();
      services.AddSingleton<ICommandHandler<SetFloatingCommand>, SetFloatingHandler>();
      services.AddSingleton<ICommandHandler<SetFocusedWindowFloatingCommand>, SetFocusedWindowFloatingHandler>();
      services.AddSingleton<ICommandHandler<ShowAllWindowsCommand>, ShowAllWindowsHandler>();
      services.AddSingleton<ICommandHandler<ToggleFloatingCommand>, ToggleFloatingHandler>();
      services.AddSingleton<ICommandHandler<ToggleFocusedWindowFloatingCommand>, ToggleFocusedWindowFloatingHandler>();
      services.AddSingleton<ICommandHandler<ActivateWorkspaceCommand>, ActivateWorkspaceHandler>();
      services.AddSingleton<ICommandHandler<DeactivateWorkspaceCommand>, DeactivateWorkspaceHandler>();
      services.AddSingleton<ICommandHandler<DisplayWorkspaceCommand>, DisplayWorkspaceHandler>();
      services.AddSingleton<ICommandHandler<FocusWorkspaceCommand>, FocusWorkspaceHandler>();
      services.AddSingleton<ICommandHandler<MoveFocusedWindowToWorkspaceCommand>, MoveFocusedWindowToWorkspaceHandler>();

      services.AddSingleton<IEventHandler<DisplaySettingsChangedEvent>, DisplaySettingsChangedHandler>();
      services.AddSingleton<IEventHandler<WindowDestroyedEvent>, WindowDestroyedHandler>();
      services.AddSingleton<IEventHandler<WindowFocusedEvent>, WindowFocusedHandler>();
      services.AddSingleton<IEventHandler<WindowHiddenEvent>, WindowHiddenHandler>();
      services.AddSingleton<IEventHandler<WindowLocationChangedEvent>, WindowLocationChangedHandler>();
      services.AddSingleton<IEventHandler<WindowMinimizedEvent>, WindowMinimizedHandler>();
      services.AddSingleton<IEventHandler<WindowMinimizeEndedEvent>, WindowMinimizeEndedHandler>();
      services.AddSingleton<IEventHandler<WindowMovedOrResizedEvent>, WindowMovedOrResizedHandler>();
      services.AddSingleton<IEventHandler<WindowShownEvent>, WindowShownHandler>();

      return services;
    }
  }
}
