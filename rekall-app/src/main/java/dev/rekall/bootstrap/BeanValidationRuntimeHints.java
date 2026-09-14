package dev.rekall.bootstrap;

import org.springframework.aot.hint.MemberCategory;
import org.springframework.aot.hint.RuntimeHints;
import org.springframework.aot.hint.RuntimeHintsRegistrar;
import org.springframework.aot.hint.TypeReference;

import java.util.List;

/**
 * Native-image reflection hints for the Hibernate Validator built-in {@code ConstraintValidator}
 * implementations backing the {@code @NotNull}/{@code @NotBlank}/{@code @Size} annotations used on
 * the API request DTOs and domain entities. Hibernate Validator instantiates these reflectively per
 * constraint, and the reachability metadata bundled for hibernate-validator only covers its logging
 * bootstrap path, not these validators, so an unregistered one fails bean creation with
 * {@code No default constructor found}. Registering the ones actually in use keeps validation
 * working in the native image.
 */
public class BeanValidationRuntimeHints implements RuntimeHintsRegistrar {

    private static final List<String> VALIDATOR_TYPES = List.of(
            "org.hibernate.validator.internal.constraintvalidators.bv.NotNullValidator",
            "org.hibernate.validator.internal.constraintvalidators.bv.NotBlankValidator",
            "org.hibernate.validator.internal.constraintvalidators.bv.size.SizeValidatorForCharSequence");

    @Override
    public void registerHints(RuntimeHints hints, ClassLoader classLoader) {
        for (String type : VALIDATOR_TYPES) {
            hints.reflection().registerType(TypeReference.of(type),
                    MemberCategory.INVOKE_DECLARED_CONSTRUCTORS,
                    MemberCategory.INVOKE_DECLARED_METHODS,
                    MemberCategory.DECLARED_FIELDS);
        }
    }
}
