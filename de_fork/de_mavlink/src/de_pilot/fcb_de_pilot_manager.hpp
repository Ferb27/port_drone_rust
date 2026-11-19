#ifndef FCB_DE_PILOT_MANAGER_H_
#define FCB_DE_PILOT_MANAGER_H_

#include "../defines.hpp"
#include "fcb_de_pilot_change_altitude.hpp"
#include "fcb_de_pilot_idle.hpp"
#include "fcb_de_pilot_operation_base.hpp"
#include "fcb_de_pilot_stabilization.hpp"
#include "fcb_de_pilot_tracking.hpp"
#include "fcb_de_pilot_yaw_control.hpp"
#include "../fcb_facade.hpp"
#include <cstdint>
#include <string>
#include <vector>
#include <queue>

namespace de {
namespace fcb {
namespace depilot {

class CDEPilotManager {
public:
  static CDEPilotManager &getInstance() {
    static CDEPilotManager instance;
    return instance;
  }

  CDEPilotManager(CDEPilotManager const &) = delete;
  void operator=(CDEPilotManager const &) = delete;

public:
  // Manager interface
  void init();
  void updateOperations();
  void reloadParametersIfConfigChanged();

  void setActive(const bool active);

public:
  // Main Functions - These are the functions we call to trigger operations
  void do_ChangeAltitude(double target_altitude, bool enqueue = false);
  void do_Stabilize(bool enqueue = false);
  void do_Stabilize(uint64_t duration_ms, bool enqueue = false);
  void do_Tracking(bool enqueue = false);
  void do_Land(bool enqueue = false);
  void do_Wait(uint64_t duration_ms, bool enqueue = false);
  void do_SetYaw(double angle, double rate, bool is_clockwise,
                 bool is_relative);
public:
  // Getters & Setters
  inline bool getActive() const {
    return m_de_pilot_operation != DEPILOT_OP_DISABLED;
  }

  inline bool getAllowRCControl() const { return m_allow_RCControl; }

  inline void setTargetAltitude(double altitude) {
    m_target_altitude = altitude;
  }

  // Operation management
  de_fork getCurrentOperation() const;
  bool isOperationActive(de_fork operation) const;

  // Queue management
  void clearOperationQueue();
  size_t getOperationQueueSize() const;

  // Target altitude management
  double getTargetAltitude() const;

public:
  void OnFlightModeChanged();

  
private:
  CDEPilotManager() {}
  ~CDEPilotManager() = default;

private:
  // Internal helper methods
  void setOperation(de_fork operation);
  void deactivateAllOperations();
  bool isCompatibleMode();
  CDEPilotOperationBase *
  getOperationInstance(de_fork operation) const;

  // Sub-operation event dispatcher
  void emitSubOperationEvent(const CDEPilotTaskBase& task);

  // Transition policy
  bool canAdvanceFromCurrentOperation() const;
  bool isYawTargetPending() const;

  // Queue request type
  struct QueuedOperation {
    de_fork operation;
    double target_altitude; // Only used for CHANGE_ALTITUDE
    bool has_altitude_param;
  };

  de::fcb::CFCBFacade& m_fcb_facade = de::fcb::CFCBFacade::getInstance();
            
  // Current operation state
  CDEPilotOperationBase *m_operation_instance = nullptr;
  double m_target_altitude = 0.0;

  // Operation queue
  std::queue<QueuedOperation> m_operation_queue;

  bool m_allow_RCControl = true;
  bool m_de_pilot_enabled = false;
  de_fork m_de_pilot_operation = DEPILOT_OP_DISABLED;
};

} // namespace depilot
} // namespace fcb
} // namespace de

#endif // FCB_DE_PILOT_MANAGER_H_
