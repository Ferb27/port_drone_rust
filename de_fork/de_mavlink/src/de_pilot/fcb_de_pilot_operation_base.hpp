#ifndef FCB_de_pilot_operation_BASE_H_
#define FCB_de_pilot_operation_BASE_H_

#include <cstdint>
#include "fcb_de_pilot_lifecycle_base.hpp"
#include "../defines.hpp"

namespace de {
namespace fcb {
namespace depilot {

class CDEPilotOperationBase : public CDEPilotLifecycleBase {
public:
    virtual ~CDEPilotOperationBase() = default;

    // Operation-specific interface
    virtual bool isCompleted() = 0;
    virtual de_fork getOperation() { return m_my_operation; }
    virtual bool isYawSupported() const { return false; }

    // Each operation class provides its own name (can be vehicle-specific)
    virtual std::string getName() const = 0;

protected:
    // Operation identifier
    de_fork m_my_operation;
};

} // namespace depilot
} // namespace fcb
} // namespace de

#endif // FCB_m_de_pilot_operation_BASE_H_
