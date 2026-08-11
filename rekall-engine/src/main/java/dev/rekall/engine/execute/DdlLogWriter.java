package dev.rekall.engine.execute;

import dev.rekall.meta.domain.DdlLog;
import dev.rekall.meta.domain.DdlStatus;
import dev.rekall.meta.repository.DdlLogRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Propagation;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/**
 * Records what a plan attempted, on a transaction of its own.
 *
 * <p>The point of {@code REQUIRES_NEW} is the failure path. A failing plan rolls its whole
 * transaction back, and log rows written inside that transaction would vanish with it, leaving
 * no trace of the attempt. Writing them on a separate connection means the record of a failed
 * migration survives the rollback that undid it.
 */
@Component
@RequiredArgsConstructor
public class DdlLogWriter {

    private final DdlLogRepository ddlLogs;

    /**
     * @param executed statements that ran before the failure and were undone by the rollback
     * @param failed the statement that raised, or {@code null} if the failure came later
     */
    @Transactional(propagation = Propagation.REQUIRES_NEW)
    public void recordFailure(UUID planId, List<String> executed, String failed, String error) {
        int sequence = 0;
        for (String statement : executed) {
            ddlLogs.save(new DdlLog(planId, sequence++, statement, DdlStatus.ROLLED_BACK, null));
        }
        if (failed != null) {
            ddlLogs.save(new DdlLog(planId, sequence, failed, DdlStatus.FAILED, error));
        }
    }

    /**
     * Written inside the caller's transaction on purpose: on the success path the log and the
     * schema change must commit together, or the rebuild recipe would not match the schema.
     */
    @Transactional(propagation = Propagation.MANDATORY)
    public void recordSuccess(UUID planId, List<String> executed) {
        int sequence = 0;
        for (String statement : executed) {
            ddlLogs.save(new DdlLog(planId, sequence++, statement, DdlStatus.APPLIED, null));
        }
    }
}
